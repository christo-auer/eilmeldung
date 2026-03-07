mod parse;
mod search_term;
mod sort_order;

pub mod prelude {
    pub use super::parse::{QueryParseError, QueryToken, strip_first_and_last};
    pub use super::search_term::{SearchTerm, to_search_term};
    pub use super::sort_order::{SortDirection, SortKey, SortOrder, SortOrderParseError};
    pub use super::{ArticleQuery, ArticleQueryContext, AugmentedArticleFilter};
}

use crate::prelude::*;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use getset::Getters;
use news_flash::models::{
    Article, ArticleFilter, ArticleID, Category, Feed, FeedID, Marked, Read, Tag, TagID,
};

#[derive(Clone, Debug)]
pub(super) enum QueryAtom {
    True,
    Read(Read),
    Marked(Marked),
    Feed(SearchTerm),
    Category(SearchTerm),
    Title(SearchTerm),
    Summary(SearchTerm),
    Author(SearchTerm),
    FeedUrl(SearchTerm),
    FeedWebUrl(SearchTerm),
    All(SearchTerm),
    Tag(Vec<String>),
    Tagged,
    Flagged,
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
        category: Option<&Category>,
        tags: Option<&HashSet<String>>,
        last_sync: &DateTime<Utc>,
        flagged_articles: &HashSet<ArticleID>,
    ) -> bool {
        match self {
            QueryClause::Id(query_atom) => {
                query_atom.test(article, feed, category, tags, last_sync, flagged_articles)
            }
            QueryClause::Not(query_atom) => {
                !query_atom.test(article, feed, category, tags, last_sync, flagged_articles)
            }
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

pub struct ArticleQueryContext<'a> {
    pub feed_map: &'a HashMap<FeedID, Feed>,
    pub category_for_feed: &'a HashMap<FeedID, Category>,
    pub tags_for_article: &'a HashMap<ArticleID, Vec<TagID>>,
    pub tag_map: &'a HashMap<TagID, Tag>,
    pub last_sync: &'a DateTime<Utc>,
    pub flagged: &'a HashSet<ArticleID>,
}

impl ArticleQuery {
    #[inline(always)]
    pub fn filter(&self, articles: &[Article], context: &ArticleQueryContext) -> Vec<Article> {
        articles
            .iter()
            .filter(|article| self.test(article, context))
            .cloned()
            .collect::<Vec<Article>>()
    }

    #[inline(always)]
    pub fn test(&self, article: &Article, context: &ArticleQueryContext) -> bool {
        let feed = context.feed_map.get(&article.feed_id);

        let category = context.category_for_feed.get(&article.feed_id);

        let tags = context
            .tags_for_article
            .get(&article.article_id)
            .map(|tag_ids| {
                tag_ids
                    .iter()
                    .filter_map(|tag_id| {
                        context.tag_map.get(tag_id).map(|tag| tag.label.to_string())
                    })
                    .collect::<HashSet<String>>()
            });

        self.query.iter().all(|query_clause| {
            query_clause.test(
                article,
                feed,
                category,
                tags.as_ref(),
                context.last_sync,
                context.flagged,
            )
        })
    }
}

impl QueryAtom {
    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed: Option<&Feed>,
        category: Option<&Category>,
        tags: Option<&HashSet<String>>,
        last_sync: &DateTime<Utc>,
        flagged_articles: &HashSet<ArticleID>,
    ) -> bool {
        use QueryAtom as A;
        match self {
            A::True => true,
            A::Read(read) => article.unread == *read,
            A::Marked(marked) => article.marked == *marked,

            A::Tagged => !tags.map(|tags| tags.is_empty()).unwrap_or(true),

            A::Flagged => flagged_articles.contains(&article.article_id),

            A::Feed(search_term)
            | A::Category(search_term)
            | A::Title(search_term)
            | A::Summary(search_term)
            | A::Author(search_term)
            | A::FeedUrl(search_term)
            | A::FeedWebUrl(search_term)
            | A::All(search_term) => self.test_string_match(search_term, article, feed, category),

            A::Tag(search_tags) => {
                let Some(tags) = tags else {
                    return false;
                };
                search_tags.iter().any(|tag| tags.contains(tag))
            }

            A::Older(date_time) => article.date < *date_time,
            A::Newer(date_time) => article.date > *date_time,
            A::SyncedAfter(date_time) => article.synced > *date_time,
            A::SyncedBefore(date_time) => article.synced < *date_time,
            A::LastSync => article.synced >= *last_sync,
        }
    }

    #[inline(always)]
    fn test_string_match(
        &self,
        search_term: &SearchTerm,
        article: &Article,
        feed: Option<&Feed>,
        category: Option<&Category>,
    ) -> bool {
        let content_string = match self {
            QueryAtom::Feed(_) => {
                let Some(feed) = feed else {
                    return false;
                };
                Some(feed.label.clone())
            }
            QueryAtom::Category(_) => {
                let Some(category) = category else {
                    return false;
                };
                Some(category.label.clone())
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

        search_term.test(&content_string)
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
