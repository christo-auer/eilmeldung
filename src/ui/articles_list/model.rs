use crate::{prelude::*, ui::articles_list::view::FilterState};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use getset::Getters;
use news_flash::models::{Article, ArticleID, Feed, FeedID, Marked, Read, Tag, TagID};

#[derive(Getters)]
#[getset(get = "pub(super)")]
pub struct ArticleListModelData {
    news_flash_utils: Arc<NewsFlashUtils>,
    articles: Vec<Article>,
    feed_map: HashMap<FeedID, Feed>,
    tags_for_article: HashMap<ArticleID, Vec<TagID>>,
    tag_map: HashMap<TagID, Tag>,
}

impl ArticleListModelData {
    pub(super) fn new(news_flash_utils: Arc<NewsFlashUtils>) -> Self {
        Self {
            news_flash_utils: news_flash_utils.clone(),

            articles: Vec::default(),
            feed_map: HashMap::default(),
            tags_for_article: HashMap::default(),
            tag_map: HashMap::default(),
        }
    }

    pub(super) async fn update(&mut self, filter_state: &FilterState) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // fill model data
        let (feeds, _) = news_flash.get_feeds()?;
        self.feed_map = NewsFlashUtils::generate_id_map(&feeds, |f| f.feed_id.clone())
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .collect();

        let (tags, taggings) = news_flash.get_tags()?;
        self.tag_map = NewsFlashUtils::generate_id_map(&tags, |t| t.tag_id.clone())
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .collect();

        self.tags_for_article = NewsFlashUtils::generate_one_to_many(
            &taggings,
            |a| a.article_id.clone(),
            |t| t.tag_id.clone(),
        );

        let position_for_tag = tags
            .iter()
            .enumerate()
            .map(|(pos, tag)| (&tag.tag_id, pos))
            .collect::<HashMap<&TagID, usize>>();

        self.tags_for_article.iter_mut().for_each(|(_, tag_ids)| {
            tag_ids.sort_by(|tag_a, tag_b| {
                position_for_tag
                    .get(tag_a)
                    .unwrap()
                    .cmp(position_for_tag.get(tag_b).unwrap())
            })
        });

        drop(news_flash);

        // apply the current filter
        self.filter_articles(filter_state).await
    }

    async fn filter_articles(&mut self, filter_state: &FilterState) -> color_eyre::Result<()> {
        let Some(augmented_article_filter) = filter_state.augmented_article_filter().as_ref()
        else {
            return Ok(());
        };

        let Some(mut article_filter) = filter_state.generate_effective_filter() else {
            return Ok(());
        };

        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // TODO make configurable
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

        Ok(())
    }

    pub(super) fn get_queried_articles(&self, query: &ArticleQuery) -> Vec<Article> {
        query.filter(
            &self.articles,
            &self.feed_map,
            &self.tags_for_article,
            &self.tag_map,
        )
    }

    pub(super) fn set_read_status(
        &mut self,
        article_ids: Vec<ArticleID>,
        read: news_flash::models::Read,
    ) -> color_eyre::Result<usize> {
        let article_ids_set: HashSet<ArticleID> = article_ids.iter().cloned().collect();

        self.news_flash_utils
            .set_article_status(article_ids.clone(), read);

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
        let article_ids_set: HashSet<ArticleID> = article_ids.iter().cloned().collect();
        self.news_flash_utils
            .set_article_marked(article_ids, marked);

        self.articles
            .iter_mut()
            .filter(|article| article_ids_set.contains(&article.article_id))
            .for_each(|article| article.marked = marked);

        Ok(article_ids_set.len())
    }
}
