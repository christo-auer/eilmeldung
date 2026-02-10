use std::{cmp::Ordering, collections::HashMap, hash::Hash, sync::Arc};

use getset::Getters;
use log::info;
use news_flash::models::{
    ArticleFilter, ArticleID, Category, CategoryID, CategoryMapping, Feed, FeedID, FeedMapping,
    PluginCapabilities, Tag, TagID, Url,
};
use ratatui::style::Color;

use crate::prelude::*;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum FeedOrCategory {
    Feed(FeedID),
    Category(CategoryID),
}

#[derive(Getters)]
#[getset(get = "pub")]
pub(super) struct FeedListModelData {
    #[getset(skip)]
    news_flash_utils: Arc<NewsFlashUtils>,

    feeds: Vec<Feed>,
    feed_map: HashMap<FeedID, Feed>,
    categories: Vec<Category>,
    category_map: HashMap<CategoryID, Category>,
    articles_for_tag: HashMap<TagID, Vec<ArticleID>>,

    unread_count_all: i64,
    unread_count_for_feed_or_category: HashMap<FeedOrCategory, i64>,
    unread_count_for_tag: HashMap<TagID, i64>,
    marked_count_for_feed_or_category: HashMap<FeedOrCategory, i64>,
    category_tree: HashMap<CategoryID, Vec<FeedOrCategory>>,
    roots: Vec<FeedOrCategory>,
    category_mapping_for_category: HashMap<CategoryID, CategoryMapping>,
    feed_mapping_for_feed: HashMap<FeedID, FeedMapping>,

    tags: Vec<Tag>,
}

impl From<CategoryID> for FeedOrCategory {
    fn from(value: CategoryID) -> Self {
        FeedOrCategory::Category(value)
    }
}

impl From<FeedID> for FeedOrCategory {
    fn from(value: FeedID) -> Self {
        FeedOrCategory::Feed(value)
    }
}

impl FeedListModelData {
    pub(super) fn new(news_flash_utils: Arc<NewsFlashUtils>) -> Self {
        Self {
            news_flash_utils: news_flash_utils.clone(),
            feeds: Vec::default(),
            feed_map: HashMap::default(),
            categories: Vec::default(),
            category_map: HashMap::default(),
            articles_for_tag: HashMap::default(),
            tags: Vec::default(),
            unread_count_all: 0,
            unread_count_for_feed_or_category: HashMap::default(),
            unread_count_for_tag: HashMap::default(),
            marked_count_for_feed_or_category: HashMap::default(),
            category_tree: HashMap::default(),
            roots: Vec::default(),
            category_mapping_for_category: HashMap::default(),
            feed_mapping_for_feed: HashMap::default(),
        }
    }

    pub(super) async fn update(&mut self) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // feeds
        let (feeds, feed_mappings) = news_flash.get_feeds()?;
        self.feed_mapping_for_feed =
            NewsFlashUtils::generate_id_map(&feed_mappings, |feed_mapping| {
                feed_mapping.feed_id.clone()
            })
            .into_iter()
            .map(|(feed_id, feed_mapping)| (feed_id, feed_mapping.clone()))
            .collect();
        self.feeds = feeds;
        self.feed_map = NewsFlashUtils::generate_id_map(&self.feeds, |f| f.feed_id.clone());

        // categories
        let (categories, category_mappings) = news_flash.get_categories()?;
        self.categories = categories;
        self.category_mapping_for_category =
            NewsFlashUtils::generate_id_map(&category_mappings, |category_mapping| {
                category_mapping.category_id.clone()
            })
            .into_iter()
            .map(|(category_id, category_mapping)| (category_id, category_mapping.clone()))
            .collect();

        self.category_map =
            NewsFlashUtils::generate_id_map(&self.categories, |c| c.category_id.clone());

        // tags
        let (tags, taggings) = news_flash.get_tags()?;
        self.tags = tags;

        self.articles_for_tag = NewsFlashUtils::generate_one_to_many(
            &taggings,
            |t| t.tag_id.clone(),
            |a| a.article_id.clone(),
        );

        self.unread_count_for_tag = HashMap::new();

        for tag in self.tags.iter() {
            let filter = ArticleFilter::tag_unread(&tag.tag_id);
            self.unread_count_for_tag.insert(
                tag.tag_id.clone(),
                news_flash.get_articles(filter)?.len() as i64,
            );
        }

        // build category/feed tree
        self.category_tree = HashMap::new();

        self.categories.iter().for_each(|category| {
            self.category_tree
                .insert(category.category_id.clone(), Vec::new());
        });

        category_mappings.iter().for_each(|category_mapping| {
            if let Some(children) = self.category_tree.get_mut(&category_mapping.parent_id) {
                children.push(FeedOrCategory::Category(
                    category_mapping.category_id.clone(),
                ));
            }
        });

        feed_mappings.iter().for_each(|feed_mapping| {
            if let Some(children) = self.category_tree.get_mut(&feed_mapping.category_id) {
                children.push(feed_mapping.feed_id.clone().into());
            }
        });

        self.category_tree.iter_mut().for_each(|(_, entries)| {
            entries.sort_by(|a, b| {
                use FeedOrCategory::*;
                match (a, b) {
                    (Category(_), Feed(_)) => Ordering::Less,
                    (Feed(_), Category(_)) => Ordering::Greater,
                    (Category(category_a), Category(category_b)) => {
                        let category_a_sort_index = self
                            .category_mapping_for_category
                            .get(category_a)
                            .map(|mapping| mapping.sort_index);
                        let category_b_sort_index = self
                            .category_mapping_for_category
                            .get(category_b)
                            .map(|mapping| mapping.sort_index);
                        category_a_sort_index.cmp(&category_b_sort_index)
                    }
                    (Feed(feed_a), Feed(feed_b)) => {
                        let feed_a_sort_index = self
                            .feed_mapping_for_feed
                            .get(feed_a)
                            .map(|mapping| mapping.sort_index);
                        let feed_b_sort_index = self
                            .feed_mapping_for_feed
                            .get(feed_b)
                            .map(|mapping| mapping.sort_index);

                        feed_a_sort_index.cmp(&feed_b_sort_index)
                    }
                }
            })
        });

        // the following has quadratic runtime but it should be fine as long as there are not
        // hundreds of categories and even more feeds
        self.roots = self
            .categories
            .iter()
            .filter(|category| {
                !category_mappings.iter().any(|category_mapping| {
                    category.category_id == category_mapping.category_id
                        && self.category_map.contains_key(&category_mapping.parent_id)
                })
            })
            .map(|category| FeedOrCategory::Category(category.category_id.to_owned()))
            .collect();
        let mut feed_roots = self
            .feeds
            .iter()
            .filter(|feed| {
                !feed_mappings.iter().any(|feed_mapping| {
                    feed_mapping.feed_id == feed.feed_id
                        && self.category_map.contains_key(&feed_mapping.category_id)
                })
            })
            .map(|feed| FeedOrCategory::Feed(feed.feed_id.to_owned()))
            .collect::<Vec<FeedOrCategory>>();
        self.roots.append(&mut feed_roots);

        // no we can build the tree structure
        self.unread_count_all = news_flash.unread_count_all()?;
        let mut unread_count_for_feed_or_category = news_flash
            .unread_count_feed_map(true)?
            .into_iter()
            .map(|(feed_id, unread)| (feed_id.into(), unread))
            .collect();
        let mut marked_count_for_feed_or_category = news_flash
            .marked_count_feed_map()?
            .into_iter()
            .map(|(feed_id, marked)| (feed_id.into(), marked))
            .collect();

        drop(news_flash);

        self.roots.iter().for_each(|feed_or_category| {
            // count unread
            if let FeedOrCategory::Category(category_id) = feed_or_category {
                Self::count_recursive(
                    category_id,
                    &self.category_tree,
                    &mut unread_count_for_feed_or_category,
                );
                //
                // count marked
                Self::count_recursive(
                    category_id,
                    &self.category_tree,
                    &mut marked_count_for_feed_or_category,
                );
            }
        });

        self.unread_count_for_feed_or_category = unread_count_for_feed_or_category;
        self.marked_count_for_feed_or_category = marked_count_for_feed_or_category;

        Ok(())
    }

    fn count_recursive(
        category_id: &CategoryID,
        tree: &HashMap<CategoryID, Vec<FeedOrCategory>>,
        count_map: &mut HashMap<FeedOrCategory, i64>,
    ) -> i64 {
        let count = tree
            .get(category_id)
            .unwrap()
            .iter()
            .map(|child| match child {
                FeedOrCategory::Category(category_id) => {
                    Self::count_recursive(category_id, tree, count_map)
                }
                feed_or_category_feed_id @ FeedOrCategory::Feed(_) => count_map
                    .get(feed_or_category_feed_id)
                    .copied()
                    .unwrap_or(0),
            })
            .sum::<i64>();
        count_map.insert(category_id.clone().into(), count);
        count
    }

    pub(super) async fn add_tag(
        &self,
        tag_title: &String,
        color: &Option<Color>,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils
            .add_tag(tag_title.to_owned(), color.to_owned());

        Ok(())
    }

    pub(super) fn remove_tag(&self, tag_id: TagID) -> color_eyre::Result<()> {
        info!("removing {}", tag_id);
        self.news_flash_utils.remove_tag(tag_id);

        Ok(())
    }

    pub(super) fn get_tag_by_label(&self, tag_label: &String) -> Option<Tag> {
        self.tags()
            .iter()
            .inspect(|tag| info!("tag: {}", tag.label))
            .find(|tag| *tag.label == *tag_label)
            .cloned()
    }

    pub(super) fn edit_tag(
        &self,
        tag_id: TagID,
        new_tag_title: String,
        color: Option<Color>,
    ) -> color_eyre::Result<()> {
        info!(
            "editing tag {:?}: name {:?} and color {:?}",
            tag_id, new_tag_title, color
        );
        self.news_flash_utils.edit_tag(tag_id, new_tag_title, color);
        Ok(())
    }

    pub(super) fn set_all_read(&self) -> color_eyre::Result<()> {
        self.news_flash_utils.set_all_read();
        Ok(())
    }

    pub(super) fn set_feed_read(&self, feed_ids: Vec<FeedID>) -> color_eyre::Result<()> {
        self.news_flash_utils.set_feed_read(feed_ids);
        Ok(())
    }

    pub(super) fn set_category_read(
        &self,
        category_ids: Vec<CategoryID>,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils.set_category_read(category_ids);
        Ok(())
    }

    pub(super) fn set_tag_read(&self, tag_ids: Vec<TagID>) -> color_eyre::Result<()> {
        self.news_flash_utils.set_tag_read(tag_ids);
        Ok(())
    }

    pub(super) fn sync(&self) -> color_eyre::Result<()> {
        self.news_flash_utils.sync();
        Ok(())
    }

    pub(super) fn add_feed(
        &self,
        url: Url,
        label: Option<String>,
        category_id: Option<CategoryID>,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils.add_feed(url, label, category_id);
        Ok(())
    }

    pub(super) fn fetch_feed(&self, feed_id: FeedID) -> color_eyre::Result<()> {
        self.news_flash_utils.fetch_feed(feed_id);
        Ok(())
    }

    pub(super) async fn add_category(
        &self,
        label: String,
        category_id: Option<CategoryID>,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils.add_category(label, category_id);
        Ok(())
    }

    pub(super) fn rename_feed(&self, feed_id: FeedID, name: String) -> color_eyre::Result<()> {
        self.news_flash_utils.rename_feed(feed_id, name);
        Ok(())
    }

    pub(super) fn remove_feed(&self, feed_id: FeedID) -> color_eyre::Result<()> {
        self.news_flash_utils.remove_feed(feed_id);
        Ok(())
    }

    pub(super) fn remove_category(
        &self,
        category_id: CategoryID,
        remove_children: bool,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils
            .remove_category(category_id, remove_children);
        Ok(())
    }

    pub(super) fn rename_category(
        &self,
        category_id: CategoryID,
        name: String,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils.rename_category(category_id, name);
        Ok(())
    }

    pub(super) async fn features(&self) -> color_eyre::Result<PluginCapabilities> {
        Ok(self
            .news_flash_utils
            .news_flash_lock
            .read()
            .await
            .features()
            .await?)
    }

    pub(super) fn change_feed_url(&self, feed_id: FeedID, url: String) -> color_eyre::Result<()> {
        self.news_flash_utils.edit_feed_url(feed_id, url);
        Ok(())
    }

    pub(super) fn move_feed(
        &self,
        from_feed_mapping: FeedMapping,
        to_feed_mapping: FeedMapping,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils
            .move_feed(from_feed_mapping, to_feed_mapping);
        Ok(())
    }

    pub(super) fn move_category(
        &self,
        category_mapping: CategoryMapping,
    ) -> color_eyre::Result<()> {
        self.news_flash_utils.move_category(category_mapping);
        Ok(())
    }
}
