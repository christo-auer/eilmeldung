use std::collections::HashMap;
use std::{cmp::Ordering, sync::Arc};

use crate::config::Config;
use news_flash::models::TagID;
use news_flash::{
    NewsFlash,
    models::{Category, CategoryID, CategoryMapping, Feed, FeedID, FeedMapping},
};
use ratatui::{
    style::{Style, Stylize},
    widgets::{StatefulWidget, Widget},
};
use ratatui::{text::Text, widgets::Scrollbar};
use tokio::sync::RwLock;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::commands::Command;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FeedListItem {
    Header(Arc<String>),
    All,
    Feed(Box<Feed>),
    Category(Box<Category>),
    Tag(Box<TagID>),
    Query(Arc<String>),
}

impl FeedListItem {
    fn to_text<'a>(
        &self,
        config: &Config,
        unread_count: Option<i64>,
        marked_count: Option<i64>,
    ) -> Text<'a> {
        use FeedListItem::*;

        let unread_count_str = unread_count.map(|c| c.to_string()).unwrap_or_default();

        let marked_count_str = marked_count.map(|c| c.to_string()).unwrap_or_default();

        let (label, mut style) = match self {
            All => (config.all_label.to_string(), config.theme.header),
            Header(header) => (header.as_ref().clone(), config.theme.header),
            Feed(feed) => (
                config.feed_label.replace("{label}", feed.label.as_str()),
                config.theme.feed,
            ),
            Category(category) => (
                config
                    .category_label
                    .replace("{label}", category.label.as_str()),
                config.theme.category,
            ),
            Tag(tag_id) => (tag_id.to_string(), config.theme.category),
            Query(query) => (query.to_string(), config.theme.category),
        };

        if let Some(unread_count) = unread_count
            && unread_count > 0
        {
            style = style.add_modifier(config.theme.unread_modifier);
        }

        Text::styled(
            label
                .replace("{unread_count}", unread_count_str.as_str())
                .replace("{marked_count}", marked_count_str.as_str()),
            style,
        )
    }
}

pub struct FeedList {
    config: Arc<Config>,
    news_flash: Arc<RwLock<NewsFlash>>,

    tree_state: TreeState<FeedListItem>,
    items: Vec<TreeItem<'static, FeedListItem>>,

    focused: bool,
}

enum FeedOrCategory<'a> {
    Feed(&'a FeedMapping),
    Category(&'a CategoryMapping),
}

impl Widget for &mut FeedList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tree = Tree::new(&self.items)
            .unwrap() // TODO error handling
            .experimental_scrollbar(Some(
                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalLeft)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(Style::new().reversed());

        StatefulWidget::render(tree, area, buf, &mut self.tree_state);
    }
}

impl FeedList {
    pub fn new(config: Arc<Config>, news_flash: Arc<RwLock<NewsFlash>>) -> Self {
        Self {
            config,
            news_flash,
            items: vec![],
            tree_state: TreeState::default(),
            focused: true,
        }
    }

    pub async fn build_tree(&mut self) -> color_eyre::Result<()> {
        let news_flash = self.news_flash.read().await;

        let (feeds, feed_mappings) = news_flash.get_feeds()?;

        let feed_map: HashMap<FeedID, Feed> = feeds
            .iter()
            .map(|feed| (feed.feed_id.clone(), feed.clone()))
            .collect();

        let (categories, category_mappings) = news_flash.get_categories()?;

        let category_map: HashMap<CategoryID, Category> = categories
            .iter()
            .map(|category| (category.clone().category_id, category.clone()))
            .collect();

        let mut tree: HashMap<CategoryID, Vec<FeedOrCategory>> = HashMap::new();

        categories.iter().for_each(|category| {
            tree.insert(category.category_id.clone(), Vec::new());
        });

        category_mappings.iter().for_each(|category_mapping| {
            if let Some(children) = tree.get_mut(&category_mapping.parent_id) {
                children.push(FeedOrCategory::Category(category_mapping));
            }
        });

        feed_mappings.iter().for_each(|feed_mapping| {
            if let Some(children) = tree.get_mut(&feed_mapping.category_id) {
                children.push(FeedOrCategory::Feed(feed_mapping));
            }
        });

        tree.iter_mut().for_each(|(_, entries)| {
            entries.sort_by(|a, b| {
                use FeedOrCategory::*;
                match (a, b) {
                    (Category(_), Feed(_)) => Ordering::Less,
                    (Feed(_), Category(_)) => Ordering::Greater,
                    (Category(category_a), Category(category_b)) => {
                        category_a.sort_index.cmp(&category_b.sort_index)
                    }
                    (Feed(feed_a), Feed(feed_b)) => feed_a.sort_index.cmp(&feed_b.sort_index),
                }
            })
        });

        let roots: Vec<Category> = categories
            .iter()
            .filter(|category| {
                category_mappings
                    .iter()
                    .any(|category_mapping| category.category_id == category_mapping.category_id)
            })
            .cloned()
            .collect();

        // no we can build the tree structure
        let unread_count_all = news_flash.unread_count_all()?;
        let unread_feed_map = news_flash.unread_count_feed_map(true)?;
        let marked_feed_map = news_flash.marked_count_feed_map()?;

        let mut unread_category_map: HashMap<CategoryID, i64> = HashMap::new();
        let mut marked_category_map: HashMap<CategoryID, i64> = HashMap::new();
        roots.iter().for_each(|category| {
            FeedList::count_categories_unread(
                &category.category_id,
                &tree,
                &unread_feed_map,
                &mut unread_category_map,
            );
            FeedList::count_categories_unread(
                &category.category_id,
                &tree,
                &marked_feed_map,
                &mut marked_category_map,
            );
        });

        self.items = vec![TreeItem::new(
            FeedListItem::All,
            FeedListItem::All.to_text(&self.config, Some(unread_count_all), None),
            feeds
                .iter()
                .map(|feed| {
                    self.map_feed_to_tree_item(feed.clone(), &unread_feed_map, &marked_feed_map)
                })
                .collect(),
        )?];

        // categories
        for root in roots.iter() {
            self.items.push(self.map_category_to_tree_item(
                root,
                &tree,
                &category_map,
                &feed_map,
                &unread_feed_map,
                &unread_category_map,
                &marked_feed_map,
                &marked_category_map,
            ));
        }

        Ok(())
    }

    fn map_feed_to_tree_item<'a>(
        &self,
        feed: Feed,
        unread_map: &HashMap<FeedID, i64>,
        marked_map: &HashMap<FeedID, i64>,
    ) -> TreeItem<'a, FeedListItem> {
        let identifier = FeedListItem::Feed(Box::new(feed.clone()));

        TreeItem::new_leaf(
            identifier.clone(),
            identifier.to_text(
                &self.config,
                unread_map.get(&feed.feed_id).copied(),
                marked_map.get(&feed.feed_id).copied(),
            ),
        )
    }

    fn count_categories_unread(
        category_id: &CategoryID,
        tree: &HashMap<CategoryID, Vec<FeedOrCategory>>,
        feed_map: &HashMap<FeedID, i64>,
        category_map: &mut HashMap<CategoryID, i64>,
    ) -> i64 {
        let count = tree
            .get(category_id)
            .unwrap()
            .iter()
            .map(|child| match child {
                FeedOrCategory::Category(category_mapping) => FeedList::count_categories_unread(
                    &category_mapping.category_id,
                    tree,
                    feed_map,
                    category_map,
                ),
                FeedOrCategory::Feed(feed_mapping) => {
                    *feed_map.get(&feed_mapping.feed_id).unwrap_or(&0)
                }
            })
            .sum::<i64>();
        category_map.insert(category_id.clone(), count);
        count
    }

    #[allow(clippy::too_many_arguments)] // yes, yes, I know
    fn map_category_to_tree_item<'a>(
        &self,
        category: &Category,
        tree: &HashMap<CategoryID, Vec<FeedOrCategory>>,
        category_map: &HashMap<CategoryID, Category>,
        feed_map: &HashMap<FeedID, Feed>,
        unread_feed_map: &HashMap<FeedID, i64>,
        unread_category_map: &HashMap<CategoryID, i64>,
        marked_feed_map: &HashMap<FeedID, i64>,
        marked_category_map: &HashMap<CategoryID, i64>,
    ) -> TreeItem<'a, FeedListItem> {
        let mut children: Vec<TreeItem<'a, FeedListItem>> = Vec::new();

        for child in tree.get(&category.category_id).unwrap_or(&Vec::new()) {
            children.push(match child {
                FeedOrCategory::Category(category_mapping) => {
                    let child_category = category_map.get(&category_mapping.category_id).unwrap();
                    self.map_category_to_tree_item(
                        child_category,
                        tree,
                        category_map,
                        feed_map,
                        unread_feed_map,
                        unread_category_map,
                        marked_feed_map,
                        marked_category_map,
                    )
                }

                FeedOrCategory::Feed(feed_mapping) => {
                    let feed = feed_map.get(&feed_mapping.feed_id).unwrap();
                    self.map_feed_to_tree_item((*feed).clone(), unread_feed_map, marked_feed_map)
                }
            });
        }

        let identifier = FeedListItem::Category(Box::new(category.clone()));
        let unread_category = unread_category_map.get(&category.category_id).copied();
        let marked_category = marked_category_map.get(&category.category_id).copied();
        TreeItem::new(
            identifier.clone(),
            identifier.to_text(&self.config, unread_category, marked_category),
            children,
        )
        .unwrap()
    }

    pub fn process_command(&mut self, command: Command) -> Option<Vec<Command>> {
        use Command::*;
        match command {
            NavigateUp if self.focused => self.tree_state.key_up(),
            NavigateDown if self.focused => self.tree_state.key_down(),
            NavigateFirst if self.focused => self.tree_state.select_first(),
            NavigateLast if self.focused => self.tree_state.select_last(),
            NavigateLeft if self.focused => self.tree_state.key_left(),
            NavigateRight if self.focused => self.tree_state.key_right(),
            NavigatePageDown if self.focused => self
                .tree_state
                .scroll_down(self.config.input_config.scroll_amount),
            NavigatePageUp if self.focused => self
                .tree_state
                .scroll_up(self.config.input_config.scroll_amount),
            _ => true,
        };

        None
    }
}
