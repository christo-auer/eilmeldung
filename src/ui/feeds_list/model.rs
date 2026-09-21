use std::{cmp::Ordering, collections::HashMap, hash::Hash, str::FromStr, sync::Arc};

use getset::Getters;
use log::info;
use news_flash::{
    NewsFlash,
    models::{
        ArticleFilter, ArticleID, Category, CategoryID, CategoryMapping, Feed, FeedID, FeedMapping,
        NEWSFLASH_TOPLEVEL, PluginCapabilities, Read, Tag, TagID, Url,
    },
};
use ratatui::style::Color;

use crate::prelude::*;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum FeedOrCategory {
    Feed(FeedID),
    Category(CategoryID),
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct FeedListModelData {
    #[getset(skip)]
    news_flash_utils: Arc<NewsFlashUtils>,

    feeds: Vec<Feed>,
    feed_map: HashMap<FeedID, Feed>,
    categories: Vec<Category>,
    category_for_category_id: HashMap<CategoryID, Category>,
    articles_for_tag_id: HashMap<TagID, Vec<ArticleID>>,
    category_tree: HashMap<CategoryID, Vec<FeedOrCategory>>,
    roots: Vec<FeedOrCategory>,
    category_mapping_for_category_id: HashMap<CategoryID, CategoryMapping>,
    feed_mapping_for_feed_id: HashMap<FeedID, FeedMapping>,

    unread_count_all: i64,
    unread_count_for_feed_or_category: HashMap<FeedOrCategory, i64>,
    unread_count_for_tag: HashMap<TagID, i64>,
    unread_count_for_query: HashMap<LabeledQuery, i64>,
    marked_count_for_feed_or_category: HashMap<FeedOrCategory, i64>,

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
            category_for_category_id: HashMap::default(),
            articles_for_tag_id: HashMap::default(),
            tags: Vec::default(),
            unread_count_all: 0,
            unread_count_for_feed_or_category: HashMap::default(),
            unread_count_for_tag: HashMap::default(),
            unread_count_for_query: HashMap::default(),
            marked_count_for_feed_or_category: HashMap::default(),
            category_tree: HashMap::default(),
            roots: Vec::default(),
            category_mapping_for_category_id: HashMap::default(),
            feed_mapping_for_feed_id: HashMap::default(),
        }
    }

    pub(super) async fn update(&mut self, config: &Config) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // feeds
        (self.feeds, self.feed_map, self.feed_mapping_for_feed_id) =
            NewsFlashUtils::get_feeds(&news_flash)?;

        // categories
        (
            self.categories,
            self.category_for_category_id,
            self.category_mapping_for_category_id,
        ) = NewsFlashUtils::get_categories(&news_flash)?;

        let parent_category_for_feed_id = NewsFlashUtils::get_parent_category_id_for_feed_id(
            &self.category_for_category_id,
            &self.feed_mapping_for_feed_id,
        );

        let parent_category_for_category_id =
            NewsFlashUtils::get_parent_category_id_for_category_id(
                &self.categories,
                &self.category_mapping_for_category_id,
            );

        // tags
        let (tags, tag_for_tag_id, tagging_for_tag_id) = NewsFlashUtils::get_tags(&news_flash)?;
        self.tags = tags;
        self.articles_for_tag_id = NewsFlashUtils::get_articles_for_tag_id(&tagging_for_tag_id);

        self.unread_count_for_tag = self.update_unread_count_for_tags(&news_flash)?;

        // build category/feed tree
        self.category_tree.clear();

        parent_category_for_category_id
            .iter()
            .for_each(|(category_id, parent_category_id)| {
                self.category_tree
                    .entry(parent_category_id.to_owned())
                    .or_default()
                    .push(category_id.clone().into())
            });

        parent_category_for_feed_id
            .iter()
            .for_each(|(feed_id, parent_category)| {
                self.category_tree
                    .entry(parent_category.category_id.to_owned())
                    .or_default()
                    .push(feed_id.clone().into());
            });

        self.feeds.sort_by(|feed_a, feed_b| {
            feed_a
                .label
                .to_uppercase()
                .cmp(&feed_b.label.to_uppercase())
        });

        self.category_tree.iter_mut().for_each(|(_, entries)| {
            entries.sort_by(|a, b| {
                use FeedOrCategory::*;
                match (a, b) {
                    (Category(_), Feed(_)) => Ordering::Less,
                    (Feed(_), Category(_)) => Ordering::Greater,
                    (Category(category_a), Category(category_b)) => {
                        let category_a_sort_index = self
                            .category_mapping_for_category_id
                            .get(category_a)
                            .map(|mapping| mapping.sort_index);
                        let category_b_sort_index = self
                            .category_mapping_for_category_id
                            .get(category_b)
                            .map(|mapping| mapping.sort_index);
                        category_a_sort_index.cmp(&category_b_sort_index)
                    }
                    (Feed(feed_a), Feed(feed_b)) => {
                        let feed_a_sort_index = self
                            .feed_mapping_for_feed_id
                            .get(feed_a)
                            .map(|mapping| mapping.sort_index);
                        let feed_b_sort_index = self
                            .feed_mapping_for_feed_id
                            .get(feed_b)
                            .map(|mapping| mapping.sort_index);

                        feed_a_sort_index.cmp(&feed_b_sort_index)
                    }
                }
            })
        });

        self.roots = self
            .categories
            .iter()
            .filter(|category| {
                parent_category_for_category_id
                    .get(&category.category_id)
                    .map(|parent_category_id| parent_category_id == &*NEWSFLASH_TOPLEVEL)
                    .unwrap_or(false)
            })
            .map(|category| category.category_id.to_owned().into())
            .collect();

        let mut feed_roots = self
            .feeds
            .iter()
            .filter(|feed| !parent_category_for_feed_id.contains_key(&feed.feed_id))
            .map(|feed| feed.feed_id.to_owned().into())
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

        // compute unread count for queries
        if config.feed_list_query_compute_unread {
            for query in config
                .feed_list
                .iter()
                .filter(|item| matches!(item, FeedListContentIdentifier::Query(..)))
            {
                if let FeedListContentIdentifier::Query(labeled_query) = query {
                    let filter = AugmentedArticleFilter::from_str(&labeled_query.query)?;

                    let articles = filter.article_query.filter(
                        &news_flash.get_articles(filter.article_filter.to_owned())?,
                        &ArticleQueryContext {
                            feed_for_feed_id: &self.feed_map,
                            parent_category_for_feed_id: &parent_category_for_feed_id,
                            tags_for_article_id: &NewsFlashUtils::get_tags_for_article(
                                &tagging_for_tag_id,
                            ),
                            tag_for_tag_id: &tag_for_tag_id,
                            last_sync: &news_flash.last_sync().await,
                            flagged: &Default::default(),
                        },
                    );

                    let unread_count = articles
                        .iter()
                        .filter(|article| matches!(article.unread, Read::Unread))
                        .count() as i64;

                    self.unread_count_for_query
                        .insert(labeled_query.to_owned(), unread_count);
                };
            }
        }

        drop(news_flash);

        Ok(())
    }

    fn update_unread_count_for_tags(
        &self,
        news_flash: &NewsFlash,
    ) -> color_eyre::Result<HashMap<TagID, i64>> {
        let mut unread_count_for_tag = HashMap::new();

        for tag in self.tags.iter() {
            let filter = ArticleFilter::tag_unread(&tag.tag_id);
            unread_count_for_tag.insert(
                tag.tag_id.clone(),
                news_flash.get_articles(filter)?.len() as i64,
            );
        }

        Ok(unread_count_for_tag)
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

    pub(super) fn set_all_read(&mut self) -> color_eyre::Result<()> {
        self.news_flash_utils.set_all_read();
        self.unread_count_for_tag.clear();
        self.unread_count_for_feed_or_category.clear();
        self.unread_count_for_query.clear();
        Ok(())
    }

    pub(super) fn set_feed_read(&mut self, feed_id: FeedID) -> color_eyre::Result<()> {
        self.unread_count_for_feed_or_category
            .entry(FeedOrCategory::Feed(feed_id.to_owned()))
            .and_modify(|entry| *entry = 0);
        self.news_flash_utils.set_feed_read(feed_id);
        Ok(())
    }

    pub(super) fn set_category_read(&mut self, category_id: CategoryID) -> color_eyre::Result<()> {
        self.set_read_count_zero_recursively(&FeedOrCategory::Category(category_id.to_owned()));
        self.news_flash_utils.set_category_read(category_id);
        Ok(())
    }

    fn set_read_count_zero_recursively(&mut self, entry: &FeedOrCategory) {
        if let FeedOrCategory::Category(category_id) = &entry
            && let Some(children) = self.category_tree().get(category_id).cloned()
        {
            children
                .iter()
                .for_each(|child| self.set_read_count_zero_recursively(child));
        }

        if let Some(count) = self.unread_count_for_feed_or_category.get_mut(entry) {
            *count = 0;
        }
    }

    pub(super) fn set_tag_read(&mut self, tag_id: TagID) -> color_eyre::Result<()> {
        self.unread_count_for_tag
            .entry(tag_id.to_owned())
            .and_modify(|entry| *entry = 0);

        self.news_flash_utils.set_tag_read(tag_id);
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

    pub(super) async fn sort(&self) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;
        news_flash.sort_alphabetically().await?;
        Ok(())
    }
}
