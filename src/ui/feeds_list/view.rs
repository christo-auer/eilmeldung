use super::model::FeedOrCategory;
use crate::prelude::*;
use crate::ui::feeds_list::model::FeedListModelData;
use std::sync::Arc;

use getset::{Getters, MutGetters};
use news_flash::models::TagID;
use news_flash::models::{Category, Feed};
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
        // let previously_selected = self.get_selected_path();
        //
        // {
        // feeds under all
        self.tree_items = vec![TreeItem::new(
            FeedListItem::All,
            FeedListItem::All.to_text(&config, Some(*model_data.unread_count_all()), None),
            model_data
                .feeds()
                .iter()
                .map(|feed| self.map_feed_to_tree_item(&config, feed, model_data))
                .collect(),
        )?];

        // categories
        for root in model_data.roots().iter() {
            self.tree_items
                .push(self.map_category_to_tree_item(&config, root, model_data));
        }

        // tags
        let tag_ids: Vec<TagID> = model_data
            .tags()
            .iter()
            .map(|tag| tag.tag_id.clone())
            .collect();
        let tags_item = FeedListItem::Tags(tag_ids);
        let tag_item_text = tags_item.to_text(&config, None, None);
        //
        self.tree_items.push(TreeItem::new(
            tags_item,
            tag_item_text,
            model_data
                .tags()
                .iter()
                .map(|tag| {
                    let tag_item = FeedListItem::Tag(Box::new(tag.clone()));

                    // TODO this is ugly => refactor
                    let tag_item_text = tag_item.to_text(
                        &config,
                        model_data.unread_count_for_tag().get(&tag.tag_id).copied(),
                        None,
                    );
                    TreeItem::new_leaf(tag_item, tag_item_text)
                })
                .collect(),
        )?);

        // queries
        config.queries.iter().for_each(|labeled_query| {
            let query_item = FeedListItem::Query(Box::new(labeled_query.clone()));
            let query_item_text = query_item.to_text(&config, None, None);
            self.tree_items
                .push(TreeItem::new_leaf(query_item, query_item_text))
        });
        // }
        //
        // if !previously_selected.is_empty() {
        //     self.select_entry(previously_selected)?;
        // } else {
        //     self.select_entry(vec![FeedListItem::All])?;
        // }

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
                    self.map_category_to_tree_item(&config, child_category, model_data)
                }

                FeedOrCategory::Feed(feed_id) => {
                    let feed = model_data.feed_map().get(feed_id).unwrap();
                    self.map_feed_to_tree_item(&config, feed, model_data)
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
        let text = identifier.to_text(&config, unread_category, marked_category);
        TreeItem::new(identifier, text, children).unwrap()
    }
}
