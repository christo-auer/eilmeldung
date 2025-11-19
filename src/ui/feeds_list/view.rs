use super::model::FeedOrCategory;
use crate::prelude::*;
use crate::ui::feeds_list::model::FeedListModelData;

use getset::{Getters, MutGetters};
use news_flash::models::{Category, Feed};
use news_flash::models::{PluginCapabilities, TagID};
use ratatui::widgets::Scrollbar;
use ratatui::widgets::{Block, Borders};
use ratatui::{
    style::{Style, Stylize},
    widgets::{StatefulWidget, Widget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::feed_list_item::FeedListItem;

#[derive(Getters, MutGetters, Default)]
pub struct FeedListViewData {
    #[getset(get = "pub", get_mut = "pub")]
    tree_state: TreeState<FeedListItem>,
    #[getset(get = "pub")]
    tree_items: Vec<TreeItem<'static, FeedListItem>>,
}

impl Widget for &mut FeedList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = if self.is_focused {
            Style::new().reversed()
        } else {
            Style::new().underlined()
        };

        let tree_items = self.view_data.tree_items().clone();
        let tree = Tree::new(&tree_items)
            .unwrap() // TODO error handling
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(if self.is_focused {
                        self.config.theme.focused_border_style
                    } else {
                        self.config.theme.border_style
                    }),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalLeft)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(highlight_style);

        StatefulWidget::render(tree, area, buf, self.view_data.tree_state_mut());
    }
}

impl FeedListViewData {
    pub async fn update(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
    ) -> color_eyre::Result<()> {
        self.tree_items = Vec::new();

        for item in config.feed_list.iter() {
            use FeedListContentIdentifier::*;
            match item {
                Feeds(item_type) => self.add_feeds_item(config, model_data, item_type)?,
                Categories(item_type) => self.add_categories_item(config, model_data, item_type)?,
                Tags(item_type) => self.add_tags_item(config, model_data, item_type).await?,
                Query(labeled_query) => self.add_query_item(config, labeled_query),
            }
        }

        Ok(())
    }

    fn add_query_item(&mut self, config: &Config, labeled_query: &LabeledQuery) {
        // queries
        let query_item = FeedListItem::Query(Box::new(labeled_query.clone()));
        let query_item_text = query_item.to_text(config, None, None);
        self.tree_items
            .push(TreeItem::new_leaf(query_item, query_item_text));
    }

    fn add_categories_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
    ) -> color_eyre::Result<()> {
        let mut root_items = Vec::new();

        for root in model_data.roots().iter() {
            root_items.push(self.map_category_to_tree_item(config, root, model_data));
        }

        match item_type {
            FeedListItemType::Tree => {
                let categories_item = FeedListItem::Categories;
                let categories_text = categories_item.to_text(config, None, None);
                self.tree_items
                    .push(TreeItem::new(categories_item, categories_text, root_items)?);
            }
            FeedListItemType::List => {
                self.tree_items.append(&mut root_items);
            }
        }
        Ok(())
    }

    async fn add_tags_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
    ) -> Result<(), color_eyre::eyre::Error> {
        if model_data
            .features()
            .await?
            .contains(PluginCapabilities::SUPPORT_TAGS)
        {
            let tag_ids: Vec<TagID> = model_data
                .tags()
                .iter()
                .map(|tag| tag.tag_id.clone())
                .collect();

            let mut children = model_data
                .tags()
                .iter()
                .map(|tag| self.gen_tag_item(config, model_data, tag.to_owned()))
                .collect::<Vec<TreeItem<_>>>();

            match item_type {
                FeedListItemType::List => self.tree_items.append(&mut children),
                FeedListItemType::Tree => {
                    let tags_item = FeedListItem::Tags(tag_ids);
                    let tag_item_text = tags_item.to_text(config, None, None);

                    let tags_tree_item = TreeItem::new(tags_item, tag_item_text, children)?;
                    self.tree_items.push(tags_tree_item);
                }
            }
        }
        Ok(())
    }

    fn add_feeds_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
    ) -> Result<(), color_eyre::eyre::Error> {
        let mut children = model_data
            .feeds()
            .iter()
            .map(|feed| self.map_feed_to_tree_item(config, feed, model_data))
            .collect();
        match item_type {
            FeedListItemType::List => self.tree_items.append(&mut children),
            FeedListItemType::Tree => self.tree_items.push(TreeItem::new(
                FeedListItem::All,
                FeedListItem::All.to_text(config, Some(*model_data.unread_count_all()), None),
                children,
            )?),
        }
        Ok(())
    }

    fn map_feed_to_tree_item<'a>(
        &self,
        config: &Config,
        feed: &Feed,
        model_data: &FeedListModelData,
    ) -> TreeItem<'a, FeedListItem> {
        let identifier = FeedListItem::Feed(Box::new(feed.clone()));
        let identifier_text = identifier.to_text(
            config,
            model_data
                .unread_count_for_feed_or_category()
                .get(&FeedOrCategory::Feed(feed.feed_id.clone()))
                .copied(),
            model_data
                .marked_count_for_feed_or_category()
                .get(&FeedOrCategory::Feed(feed.feed_id.clone()))
                .copied(),
        );

        TreeItem::new_leaf(identifier, identifier_text)
    }

    #[allow(clippy::too_many_arguments)] // yes, yes, I know
    fn map_category_to_tree_item<'a>(
        &self,
        config: &Config,
        category: &Category,
        model_data: &FeedListModelData,
    ) -> TreeItem<'a, FeedListItem> {
        let mut children: Vec<TreeItem<'a, FeedListItem>> = Vec::new();

        for child in model_data
            .category_tree()
            .get(&category.category_id)
            .unwrap_or(&Vec::new())
        {
            children.push(match child {
                FeedOrCategory::Category(category_id) => {
                    let child_category = model_data.category_map().get(category_id).unwrap();
                    self.map_category_to_tree_item(config, child_category, model_data)
                }

                FeedOrCategory::Feed(feed_id) => {
                    let feed = model_data.feed_map().get(feed_id).unwrap();
                    self.map_feed_to_tree_item(config, feed, model_data)
                }
            });
        }

        let identifier = FeedListItem::Category(Box::new(category.clone()));
        let unread_category = model_data
            .unread_count_for_feed_or_category()
            .get(&category.category_id.clone().into())
            .copied();
        let marked_category = model_data
            .marked_count_for_feed_or_category()
            .get(&category.category_id.clone().into())
            .copied();
        let text = identifier.to_text(config, unread_category, marked_category);
        TreeItem::new(identifier, text, children).unwrap()
    }

    fn gen_tag_item(
        &self,
        config: &Config,
        model_data: &FeedListModelData,
        tag: news_flash::models::Tag,
    ) -> TreeItem<'static, FeedListItem> {
        let count = model_data.unread_count_for_tag().get(&tag.tag_id).copied();
        let tag_item = FeedListItem::Tag(Box::new(tag));
        let tag_item_text = tag_item.to_text(config, count, None);
        TreeItem::new_leaf(tag_item, tag_item_text)
    }
}
