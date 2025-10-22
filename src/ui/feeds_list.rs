use std::collections::HashMap;
use std::str::FromStr;
use std::{cmp::Ordering, sync::Arc};

use crate::app::AppState;
use crate::config::Config;
use crate::newsflash_utils::NewsFlashUtils;
use crate::ui::tooltip::{Tooltip, TooltipFlavor};
use news_flash::models::{ArticleFilter, ArticleID, Marked, Read, Tag, TagID};
use news_flash::models::{Category, CategoryID, CategoryMapping, Feed, FeedID, FeedMapping};
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders};
use ratatui::{
    style::{Style, Stylize},
    widgets::{StatefulWidget, Widget},
};
use ratatui::{text::Text, widgets::Scrollbar};
use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::commands::{Command, Event, Message, MessageReceiver};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FeedListItem {
    All,
    Feed(Box<Feed>),
    Category(Box<Category>),
    Tags(Vec<TagID>),
    Tag(Box<Tag>),
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
            Tags(_) => (config.tags_label.to_string(), config.theme.header),
            Tag(tag) => {
                let mut style = config.theme.tag;
                
                // Use our utility function for color if tag doesn't have a custom color
                let tag_color = if let Some(color_str) = tag.color.clone()
                    && let Ok(custom_color) = Color::from_str(color_str.as_str())
                {
                    custom_color
                } else {
                    // Use our utility function for semantic colors
                    let color_str = NewsFlashUtils::get_tag_color(&tag.label);
                    Color::from_str(color_str).unwrap_or(Color::Gray)
                };
                style = style.fg(tag_color);
                
                // Get semantic icon for the tag
                let icon = NewsFlashUtils::get_tag_icon(&tag.label);
                
                // Replace the icon in the label format
                let label_with_icon = config.tag_label
                    .replace("{label}", &format!("{} {}", icon, tag.label));
                
                (label_with_icon, style)
            }

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

    fn to_tooltip(&self, _config: &Config) -> String {
        use FeedListItem::*;
        match self {
            All => "all feeds".to_string(),
            Category(category) => format!("Category: {}", category.label).to_string(),
            Feed(feed) => {
                format!(
                    "Feed: {} ({})",
                    feed.label,
                    feed.website
                        .clone()
                        .map(|url| url.to_string())
                        .unwrap_or("no url".into())
                )
            }
            Tags(_) => "all tagged articles".to_string(),
            Tag(tag) => format!("Tag: {}", tag.label),
            Query(query) => format!("Query: {}", query),
        }
    }
}

impl From<FeedListItem> for ArticleFilter {
    fn from(value: FeedListItem) -> Self {
        use FeedListItem::*;
        match value {
            All => ArticleFilter::default(),
            Feed(feed) => ArticleFilter {
                feeds: vec![feed.feed_id].into(),
                ..Default::default()
            },
            Category(category) => ArticleFilter {
                categories: vec![category.category_id].into(),
                ..Default::default()
            },
            Tags(tag_ids) => ArticleFilter {
                tags: Some(tag_ids),
                ..Default::default()
            },
            Tag(tag) => ArticleFilter {
                tags: vec![tag.tag_id].into(),
                ..Default::default()
            },
            Query(query) => ArticleFilter {
                search_term: Some((*query).clone()),
                ..Default::default()
            },
        }
    }
}

pub struct FeedList {
    config: Arc<Config>,
    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    tree_state: TreeState<FeedListItem>,
    items: Vec<TreeItem<'static, FeedListItem>>,

    is_focused: bool,
}

enum FeedOrCategory<'a> {
    Feed(&'a FeedMapping),
    Category(&'a CategoryMapping),
}

impl Widget for &mut FeedList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = if self.is_focused {
            Style::new().reversed()
        } else {
            Style::new().underlined()
        };

        let tree = Tree::new(&self.items)
            .unwrap() // TODO error handling
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(self.config.theme.border_style),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalLeft)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(highlight_style);

        StatefulWidget::render(tree, area, buf, &mut self.tree_state);
    }
}

impl FeedList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config,
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            items: vec![],
            tree_state: TreeState::default(),
            is_focused: true,
        }
    }

    pub async fn build_tree(&mut self) -> color_eyre::Result<()> {
        let previously_selected = self.get_selected_path();

        {
            let news_flash = self.news_flash_utils.news_flash_lock.read().await;

            // feeds
            let (feeds, feed_mappings) = news_flash.get_feeds()?;
            let feed_map = NewsFlashUtils::generate_id_map(&feeds, |f| f.feed_id.clone());

            // categories
            let (categories, category_mappings) = news_flash.get_categories()?;

            let category_map =
                NewsFlashUtils::generate_id_map(&categories, |c| c.category_id.clone());

            // tags
            let (tags, taggings) = news_flash.get_tags()?;

            let articles_for_tag: HashMap<TagID, Vec<ArticleID>> =
                NewsFlashUtils::generate_one_to_many(
                    &taggings,
                    |t| t.tag_id.clone(),
                    |a| a.article_id.clone(),
                );

            // build category/feed tree
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
                    category_mappings.iter().any(|category_mapping| {
                        category.category_id == category_mapping.category_id
                    })
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

            // feeds under all
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

            // tags
            let tag_ids: Vec<TagID> = tags.iter().map(|tag| tag.tag_id.clone()).collect();
            let tags_item = FeedListItem::Tags(tag_ids);

            self.items.push(TreeItem::new(
                tags_item.clone(),
                tags_item.to_text(&self.config, None, None),
                tags.iter()
                    .map(|tag| {
                        let tag_item = FeedListItem::Tag(Box::new(tag.clone()));

                        // TODO this is ugly => refactor
                        let (unread_count, marked_count) = match articles_for_tag.get(&tag.tag_id) {
                            None => (None, None),
                            Some(articles) => (
                                Some(
                                    articles
                                        .iter()
                                        .filter(|article_id| {
                                            news_flash
                                                .get_article(article_id)
                                                .map(|article| article.unread)
                                                .unwrap_or(Read::Read)
                                                == Read::Unread
                                        })
                                        .count() as i64,
                                ),
                                Some(
                                    articles
                                        .iter()
                                        .filter(|article_id| {
                                            news_flash
                                                .get_article(article_id)
                                                .map(|article| article.marked)
                                                .unwrap_or(Marked::Unmarked)
                                                == Marked::Marked
                                        })
                                        .count() as i64,
                                ),
                            ),
                        };
                        TreeItem::new(
                            tag_item.clone(),
                            tag_item.to_text(&self.config, unread_count, marked_count),
                            [].into(),
                        )
                        .unwrap()
                    })
                    .collect(),
            )?);
        }

        self.select_entry(previously_selected)?;

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

    fn update_tooltip(&self, now_selected: Option<FeedListItem>) -> color_eyre::Result<()> {
        if let Some(item) = now_selected {
            self.message_sender
                .send(Message::Event(Event::Tooltip(Tooltip::from_str(
                    item.to_tooltip(&self.config).as_str(),
                    TooltipFlavor::Info,
                ))))?;
        }

        Ok(())
    }

    fn get_selected(&self) -> Option<FeedListItem> {
        self.tree_state.selected().last().cloned()
    }

    fn get_selected_path(&self) -> Vec<FeedListItem> {
        self.tree_state.selected().to_vec()
    }

    fn select_entry(&mut self, path: Vec<FeedListItem>) -> color_eyre::Result<()> {
        if self.tree_state.select(path) {
            let now_selected = self.tree_state.selected().last().cloned();
            self.message_sender
                .send(Message::Event(Event::ArticlesSelected(
                    now_selected.unwrap().into(),
                )))?;
        }

        Ok(())
    }

    fn generate_articles_selected_command(&self) -> color_eyre::Result<()> {
        if let Some(selected) = self.get_selected() {
            self.message_sender
                .send(Message::Event(Event::ArticlesSelected(selected.into())))?;
        };

        Ok(())
    }
}

impl MessageReceiver for FeedList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        // get selection before
        let selected_before_item = self.tree_state.selected().last().cloned();

        match message {
            Message::Command(NavigateUp) if self.is_focused => {
                self.tree_state.key_up();
            }
            Message::Command(NavigateDown) if self.is_focused => {
                self.tree_state.key_down();
            }
            Message::Command(NavigateFirst) if self.is_focused => {
                self.tree_state.select_first();
            }
            Message::Command(NavigateLast) if self.is_focused => {
                self.tree_state.select_last();
            }
            Message::Command(NavigateLeft) if self.is_focused => {
                self.tree_state.key_left();
            }
            Message::Command(NavigateRight) if self.is_focused => {
                self.tree_state.key_right();
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.tree_state
                    .scroll_down(self.config.input_config.scroll_amount);
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.tree_state
                    .scroll_up(self.config.input_config.scroll_amount);
            }

            Message::Event(ApplicationStarted)
            | Message::Event(AsyncSyncFinished(_))
            | Message::Event(AsyncMarkArticlesAsReadFinished) => {
                self.build_tree().await?;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::FeedSelection;
            }

            _ => (),
        };

        // get selection after
        let selected_after_item = self.tree_state.selected().last().cloned();

        if selected_before_item != selected_after_item {
            self.generate_articles_selected_command()?;
            self.update_tooltip(selected_after_item)?;
        }

        Ok(())
    }
}
