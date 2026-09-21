use crate::{prelude::*, ui::articles_list::view::FilterState};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use getset::{Getters, MutGetters};
use log::info;
use news_flash::models::{Article, ArticleID, Category, Feed, FeedID, Marked, Tag, TagID};

#[derive(Getters, MutGetters)]
#[getset(get = "pub(super)")]
pub struct ArticleListModelData {
    news_flash_utils: Arc<NewsFlashUtils>,
    articles: Vec<Article>,
    feed_for_feed_id: HashMap<FeedID, Feed>,
    parent_category_for_feed_id: HashMap<FeedID, Category>,
    tags_for_article_id: HashMap<ArticleID, Vec<TagID>>,
    tag_for_tag_id: HashMap<TagID, Tag>,
    last_sync: DateTime<Utc>,

    #[get_mut = "pub(super)"]
    flagged_articles: HashSet<ArticleID>,
}

impl ArticleListModelData {
    pub(super) fn new(news_flash_utils: Arc<NewsFlashUtils>) -> Self {
        Self {
            news_flash_utils: news_flash_utils.clone(),

            articles: Default::default(),
            feed_for_feed_id: Default::default(),
            parent_category_for_feed_id: Default::default(),
            tags_for_article_id: Default::default(),
            tag_for_tag_id: Default::default(),
            last_sync: Default::default(),
            flagged_articles: Default::default(),
        }
    }

    pub(super) async fn update(
        &mut self,
        config: &Config,
        filter_state: &FilterState,
    ) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // last sync
        self.last_sync = news_flash.last_sync().await;

        // fill model data
        let (_feeds, feed_for_feed_id, feed_mapping_for_feed_id) =
            NewsFlashUtils::get_feeds(&news_flash)?;

        self.feed_for_feed_id = feed_for_feed_id;

        let (_categories, category_for_category_id, _category_mapping_for_category_id) =
            NewsFlashUtils::get_categories(&news_flash)?;

        self.parent_category_for_feed_id = NewsFlashUtils::get_parent_category_id_for_feed_id(
            &category_for_category_id,
            &feed_mapping_for_feed_id,
        );

        let (tags, tag_for_tag_id, tagging_for_tag_id) = NewsFlashUtils::get_tags(&news_flash)?;
        self.tag_for_tag_id = tag_for_tag_id;
        self.tags_for_article_id = NewsFlashUtils::get_tags_for_article(&tagging_for_tag_id);

        let position_for_tag = tags
            .iter()
            .enumerate()
            .map(|(pos, tag)| (&tag.tag_id, pos))
            .collect::<HashMap<&TagID, usize>>();

        self.tags_for_article_id
            .iter_mut()
            .for_each(|(_, tag_ids)| {
                tag_ids.sort_by(|tag_a, tag_b| {
                    position_for_tag
                        .get(tag_a)
                        .unwrap()
                        .cmp(position_for_tag.get(tag_b).unwrap())
                })
            });

        drop(news_flash);

        // apply the current filter
        self.filter_articles(config, filter_state).await
    }

    pub(super) fn effectively_flagged_articles(&self) -> Vec<ArticleID> {
        self.flagged_articles
            .intersection(&HashSet::from_iter(
                self.articles
                    .iter()
                    .map(|article| article.article_id.to_owned()),
            ))
            .cloned()
            .collect()
    }

    async fn filter_articles(
        &mut self,
        config: &Config,
        filter_state: &FilterState,
    ) -> color_eyre::Result<()> {
        let Some(augmented_article_filter) = filter_state.augmented_article_filter().as_ref()
        else {
            return Ok(());
        };

        let Some(mut article_filter) = filter_state.generate_effective_filter() else {
            return Ok(());
        };

        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        article_filter.order_by = Some(news_flash::models::OrderBy::Published);
        article_filter.order = Some(news_flash::models::ArticleOrder::NewestFirst);

        self.articles = news_flash.get_articles(article_filter.clone())?;

        if augmented_article_filter.is_augmented() {
            self.articles = self.get_queried_articles(&augmented_article_filter.article_query);
        }

        if let Some(article_adhoc_filter) = filter_state.article_adhoc_filter().as_ref()
            && *filter_state.apply_article_adhoc_filter()
        {
            self.articles = self.get_queried_articles(article_adhoc_filter);
        }

        filter_state
            .get_effective_sort_order(config)
            .sort(&mut self.articles, &self.feed_for_feed_id);

        Ok(())
    }

    pub(super) fn get_queried_articles(&self, query: &ArticleQuery) -> Vec<Article> {
        query.filter(
            &self.articles,
            &ArticleQueryContext {
                feed_for_feed_id: self.feed_for_feed_id(),
                parent_category_for_feed_id: self.parent_category_for_feed_id(),
                tags_for_article_id: self.tags_for_article_id(),
                tag_for_tag_id: self.tag_for_tag_id(),
                last_sync: self.last_sync(),
                flagged: &self.flagged_articles,
            },
        )
    }

    pub(super) fn set_read_status(
        &mut self,
        article_ids: Vec<ArticleID>,
        read: news_flash::models::Read,
    ) -> color_eyre::Result<usize> {
        // no articles -> no changes needed
        if article_ids.is_empty() {
            return Ok(0);
        }

        let article_ids_set: HashSet<ArticleID> = article_ids.iter().cloned().collect();

        self.news_flash_utils
            .set_article_status(article_ids.clone(), read, true); // undoable

        self.articles
            .iter_mut()
            .filter(|article| article_ids_set.contains(&article.article_id))
            .for_each(|article| article.unread = read);

        Ok(article_ids_set.len())
    }

    pub(super) fn set_marked_status(
        &mut self,
        article_ids: Vec<ArticleID>,
        marked: Marked,
    ) -> color_eyre::Result<usize> {
        if article_ids.is_empty() {
            return Ok(0);
        }

        let article_ids_set: HashSet<ArticleID> = article_ids.iter().cloned().collect();
        self.news_flash_utils
            .set_article_marked(article_ids, marked, true);

        self.articles
            .iter_mut()
            .filter(|article| article_ids_set.contains(&article.article_id))
            .for_each(|article| article.marked = marked);

        Ok(article_ids_set.len())
    }

    pub(super) fn tag_articles(
        &mut self,
        article_ids: Vec<ArticleID>,
        tag_id: TagID,
    ) -> color_eyre::Result<usize> {
        if article_ids.is_empty() {
            return Ok(0);
        }

        let article_ids = article_ids
            .into_iter()
            .filter(|article_id| {
                self.tags_for_article_id
                    .get(article_id)
                    .map(|tags| !tags.contains(&tag_id))
                    .unwrap_or(true)
            })
            .collect::<Vec<ArticleID>>();
        let count = article_ids.len();
        info!("tagging {} articles with {}", count, tag_id);
        self.news_flash_utils
            .tag_articles(article_ids, tag_id.clone(), true);

        Ok(count)
    }

    pub(super) fn untag_articles(
        &mut self,
        article_ids: Vec<ArticleID>,
        tag_id: TagID,
    ) -> color_eyre::Result<usize> {
        if article_ids.is_empty() {
            return Ok(0);
        }

        let article_ids = article_ids
            .into_iter()
            .filter(|article_id| {
                self.tags_for_article_id
                    .get(article_id)
                    .map(|tags| tags.contains(&tag_id))
                    .unwrap_or(false)
            })
            .collect::<Vec<ArticleID>>();
        let count = article_ids.len();
        info!("removing tag {} from {} articles", tag_id, count);

        self.news_flash_utils
            .untag_articles(article_ids, tag_id, true);

        Ok(count)
    }
}
