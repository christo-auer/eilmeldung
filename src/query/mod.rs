mod parse;
mod sort_order;

pub mod prelude {
    pub use super::parse::{QueryParseError, QueryToken};
    pub use super::sort_order::{SortDirection, SortKey, SortOrder, SortOrderParseError};
    pub use super::{ArticleQuery, AugmentedArticleFilter};
}

use crate::prelude::*;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use getset::Getters;
use news_flash::models::{
    Article, ArticleFilter, ArticleID, Feed, FeedID, Marked, Read, Tag, TagID,
};
use regex::Regex;

#[derive(Clone, Debug)]
pub(super) enum SearchTerm {
    Verbatim(String),
    Word(String),
    Regex(Regex),
}

#[derive(Clone, Debug)]
pub(super) enum QueryAtom {
    True,
    Read(Read),
    Marked(Marked),
    Feed(SearchTerm),
    Title(SearchTerm),
    Summary(SearchTerm),
    Author(SearchTerm),
    FeedUrl(SearchTerm),
    FeedWebUrl(SearchTerm),
    All(SearchTerm),
    Tag(Vec<String>),
    Tagged,
    LastSync,
    Newer(DateTime<Utc>),
    Older(DateTime<Utc>),
    SyncedBefore(DateTime<Utc>),
    SyncedAfter(DateTime<Utc>),
}

#[derive(Clone, Debug)]
pub(super) enum QueryClause {
    Id(QueryAtom),
    Not(QueryAtom),
}

impl QueryClause {
    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed: Option<&Feed>,
        tags: Option<&HashSet<String>>,
        last_sync: &DateTime<Utc>,
    ) -> bool {
        match self {
            QueryClause::Id(query_atom) => query_atom.test(article, feed, tags, last_sync),
            QueryClause::Not(query_atom) => !query_atom.test(article, feed, tags, last_sync),
        }
    }
}

#[derive(Default, Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct ArticleQuery {
    query_string: String,
    query: Vec<QueryClause>,
    sort_order: Option<SortOrder>,
}

impl ArticleQuery {
    #[inline(always)]
    pub fn filter(
        &self,
        articles: &[Article],
        feed_map: &HashMap<FeedID, Feed>,
        tags_for_article: &HashMap<ArticleID, Vec<TagID>>,
        tag_map: &HashMap<TagID, Tag>,
        last_sync: &DateTime<Utc>,
    ) -> Vec<Article> {
        articles
            .iter()
            .filter(|article| self.test(article, feed_map, tags_for_article, tag_map, last_sync))
            .cloned()
            .collect::<Vec<Article>>()
    }

    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed_map: &HashMap<FeedID, Feed>,
        tags_for_article: &HashMap<ArticleID, Vec<TagID>>,
        tag_map: &HashMap<TagID, Tag>,
        last_sync: &DateTime<Utc>,
    ) -> bool {
        let feed = feed_map.get(&article.feed_id);

        let tags = tags_for_article.get(&article.article_id).map(|tag_ids| {
            tag_ids
                .iter()
                .filter_map(|tag_id| tag_map.get(tag_id).map(|tag| tag.label.to_string()))
                .collect::<HashSet<String>>()
        });

        self.query
            .iter()
            .all(|query_clause| query_clause.test(article, feed, tags.as_ref(), last_sync))
    }
}

impl QueryAtom {
    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed: Option<&Feed>,
        tags: Option<&HashSet<String>>,
        last_sync: &DateTime<Utc>,
    ) -> bool {
        use QueryAtom::*;
        match self {
            True => true,
            Read(read) => article.unread == *read,
            Marked(marked) => article.marked == *marked,

            Tagged => !tags.map(|tags| tags.is_empty()).unwrap_or(true),

            Feed(search_term)
            | Title(search_term)
            | Summary(search_term)
            | Author(search_term)
            | FeedUrl(search_term)
            | FeedWebUrl(search_term)
            | All(search_term) => self.test_string_match(search_term, article, feed),

            Tag(search_tags) => {
                let Some(tags) = tags else {
                    return false;
                };
                search_tags.iter().any(|tag| tags.contains(tag))
            }

            Older(date_time) => article.date < *date_time,
            Newer(date_time) => article.date > *date_time,
            SyncedAfter(date_time) => article.synced > *date_time,
            SyncedBefore(date_time) => article.synced < *date_time,
            LastSync => article.synced >= *last_sync,
        }
    }

    #[inline(always)]
    fn test_string_match(
        &self,
        search_term: &SearchTerm,
        article: &Article,
        feed: Option<&Feed>,
    ) -> bool {
        let content_string = match self {
            QueryAtom::Feed(_) => {
                let Some(feed) = feed else {
                    return false;
                };
                Some(feed.label.clone())
            }
            QueryAtom::FeedUrl(_) => {
                let Some(feed) = feed else {
                    return false;
                };
                feed.feed_url.clone().map(|url| url.to_string())
            }
            QueryAtom::FeedWebUrl(_) => {
                let Some(feed) = feed else {
                    return false;
                };
                feed.website.clone().map(|url| url.to_string())
            }
            QueryAtom::Title(_) => article.title.clone(),
            QueryAtom::Summary(_) => article.summary.clone(),
            QueryAtom::Author(_) => article.author.clone(),
            QueryAtom::All(_) => Some(format!(
                "{} {} {} {} {} {}",
                article.title.as_deref().unwrap_or_default(),
                article.summary.as_deref().unwrap_or_default(),
                article.author.as_deref().unwrap_or_default(),
                feed.as_ref()
                    .map(|feed| feed.label.as_str())
                    .unwrap_or_default(),
                feed.as_ref()
                    .map(|feed| feed
                        .feed_url
                        .as_ref()
                        .map(|url| url.to_string())
                        .unwrap_or_default())
                    .unwrap_or_default()
                    .as_str(),
                feed.as_ref()
                    .map(|feed| feed
                        .website
                        .as_ref()
                        .map(|url| url.to_string())
                        .unwrap_or_default())
                    .unwrap_or_default()
                    .as_str(),
            )),
            _ => unreachable!(),
        };

        let Some(content_string) = content_string else {
            return false;
        };

        match search_term {
            SearchTerm::Regex(regex) => regex.is_match(&content_string),
            SearchTerm::Verbatim(term) => content_string.contains(term),
            SearchTerm::Word(word) => content_string.to_lowercase().contains(&word.to_lowercase()),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AugmentedArticleFilter {
    pub article_filter: ArticleFilter,
    pub article_query: ArticleQuery,
}

impl From<ArticleFilter> for AugmentedArticleFilter {
    fn from(article_filter: ArticleFilter) -> Self {
        Self {
            article_filter,
            ..Self::default()
        }
    }
}

impl From<ArticleQuery> for AugmentedArticleFilter {
    fn from(article_query: ArticleQuery) -> Self {
        Self {
            article_query,
            ..Self::default()
        }
    }
}

impl AugmentedArticleFilter {
    pub fn new(article_filter: ArticleFilter, article_query: ArticleQuery) -> Self {
        Self {
            article_filter,
            article_query,
        }
    }

    pub fn is_augmented(&self) -> bool {
        !self.article_query.query.is_empty()
    }

    pub fn defines_scope(&self) -> bool {
        self.is_augmented()
            || self.article_filter.unread.is_some()
            || self.article_filter.marked.is_some()
    }
}
