use crate::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

use news_flash::models::{ArticleID, Marked, Read, TagID};
use news_flash::models::{Category, CategoryID, CategoryMapping, Feed, FeedID, FeedMapping};
use ratatui::widgets::Scrollbar;
use ratatui::widgets::{Block, Borders};
use ratatui::{
    style::{Style, Stylize},
    widgets::{StatefulWidget, Widget},
};
use tui_tree_widget::{Tree, TreeItem};

use super::feed_list_item::FeedListItem;

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

        StatefulWidget::render(tree, area, buf, &mut self.tree_state);
    }
}

impl FeedList {
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
                        self.map_feed_to_tree_item(feed, &unread_feed_map, &marked_feed_map)
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
            let tag_item_text = tags_item.to_text(&self.config, None, None);

            self.items.push(TreeItem::new(
                tags_item,
                tag_item_text,
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
                        let tag_item_text =
                            tag_item.to_text(&self.config, unread_count, marked_count);
                        TreeItem::new_leaf(tag_item, tag_item_text)
                    })
                    .collect(),
            )?);

            // queries
            self.config.queries.iter().for_each(|labeled_query| {
                let query_item = FeedListItem::Query(Box::new(labeled_query.clone()));
                let query_item_text = query_item.to_text(&self.config, None, None);
                self.items
                    .push(TreeItem::new_leaf(query_item, query_item_text))
            });
        }

        if !previously_selected.is_empty() {
            self.select_entry(previously_selected)?;
        } else {
            self.select_entry(vec![FeedListItem::All])?;
        }

        Ok(())
    }

    fn map_feed_to_tree_item<'a>(
        &self,
        feed: &Feed,
        unread_map: &HashMap<FeedID, i64>,
        marked_map: &HashMap<FeedID, i64>,
    ) -> TreeItem<'a, FeedListItem> {
        let identifier = FeedListItem::Feed(Box::new(feed.clone()));
        let identifier_text = identifier.to_text(
            &self.config,
            unread_map.get(&feed.feed_id).copied(),
            marked_map.get(&feed.feed_id).copied(),
        );

        TreeItem::new_leaf(identifier, identifier_text)
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
                    self.map_feed_to_tree_item(feed, unread_feed_map, marked_feed_map)
                }
            });
        }

        let identifier = FeedListItem::Category(Box::new(category.clone()));
        let unread_category = unread_category_map.get(&category.category_id).copied();
        let marked_category = marked_category_map.get(&category.category_id).copied();
        let text = identifier.to_text(&self.config, unread_category, marked_category);
        TreeItem::new(identifier, text, children).unwrap()
    }

    pub(super) fn update_tooltip(
        &self,
        now_selected: Option<&FeedListItem>,
    ) -> color_eyre::Result<()> {
        if let Some(item) = now_selected {
            tooltip(
                &self.message_sender,
                item.to_tooltip(&self.config).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    pub(super) fn get_selected(&self) -> Option<FeedListItem> {
        self.tree_state.selected().last().cloned()
    }

    fn get_selected_path(&self) -> Vec<FeedListItem> {
        self.tree_state.selected().to_vec()
    }

    fn select_entry(&mut self, path: Vec<FeedListItem>) -> color_eyre::Result<()> {
        if self.tree_state.select(path) {
            self.generate_articles_selected_command()?;
        }

        Ok(())
    }
}
