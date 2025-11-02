use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use log::trace;
use logos::Logos;
use news_flash::models::{
    Article, ArticleFilter, ArticleID, Feed, FeedID, Marked, Read, Tag, TagID,
};
use parse_datetime::parse_datetime;
use regex::Regex;

#[derive(Clone, Debug)]
enum SearchTerm {
    Verbatim(String),
    Word(String),
    Regex(Regex),
}

#[derive(Clone, Debug)]
enum QueryAtom {
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
    Newer(DateTime<Utc>),
    Older(DateTime<Utc>),
    SyncedBefore(DateTime<Utc>),
    SyncedAfter(DateTime<Utc>),
}

#[derive(Clone, Debug)]
enum QueryClause {
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
    ) -> bool {
        match self {
            QueryClause::Id(query_atom) => query_atom.test(article, feed, tags),
            QueryClause::Not(query_atom) => !query_atom.test(article, feed, tags),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ArticleQuery {
    query_string: String,
    query: Vec<QueryClause>,
}

impl ArticleQuery {
    pub fn query_string(&self) -> &str {
        self.query_string.as_str()
    }

    #[inline(always)]
    pub fn filter(
        &self,
        articles: &[Article],
        feed_map: &HashMap<FeedID, Feed>,
        tags_for_article: &HashMap<ArticleID, Vec<TagID>>,
        tag_map: &HashMap<TagID, Tag>,
    ) -> Vec<Article> {
        articles
            .iter()
            .filter(|article| self.test(article, feed_map, tags_for_article, tag_map))
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
            .all(|query_clause| query_clause.test(article, feed, tags.as_ref()))
    }
}

impl FromStr for ArticleQuery {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_query(s, &mut None)
    }
}

impl QueryAtom {
    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed: Option<&Feed>,
        tags: Option<&HashSet<String>>,
    ) -> bool {
        use QueryAtom::*;
        match self {
            Read(read) => article.unread == *read,
            Marked(marked) => article.marked == *marked,

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

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum QueryToken {
    #[token("~", priority = 2)]
    Negate,

    #[token("read", priority = 2)]
    KeyRead,

    #[token("unread", priority = 2)]
    KeyUnread,

    #[token("marked", priority = 2)]
    KeyMarked,

    #[token("unmarked", priority = 2)]
    KeyUnmarked,

    #[token("newer:")]
    KeyNewer,

    #[token("older:")]
    KeyOlder,

    #[token("syncedbefore:")]
    KeySyncedBefore,

    #[token("syncedafter:")]
    KeySyncedAfter,

    #[token("feed:")]
    KeyFeed,

    #[token("title:")]
    KeyTitle,

    #[token("summary:")]
    KeySummary,

    #[token("author:")]
    KeyAuthor,

    #[token("all:")]
    KeyAll,

    #[token("feedurl:")]
    KeyFeedUrl,

    #[token("feedweburl:")]
    KeyFeedWebUrl,

    #[token("tag:")]
    KeyTag,

    #[regex(r#""[^"\n\r\\]*(?:\\.[^"\n\r\\]*)*""#)]
    QuotedString,

    #[regex(r#"[\w]+"#, priority = 1)]
    Word,

    #[regex(r"/[^/\\]*(?:\\.[^/\\]*)*/")]
    Regex,

    #[regex(r#"#[a-zA-Z][a-zA-Z0-9]*(?:,#[a-zA-Z][a-zA-Z0-9]*)*"#)]
    TagList,
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
}

impl FromStr for AugmentedArticleFilter {
    type Err = color_eyre::Report;

    fn from_str(query: &str) -> Result<Self, Self::Err> {
        let mut article_filter = ArticleFilter::default();

        let article_query = parse_query(query, &mut Some(&mut article_filter))?;
        Ok(Self::new(article_filter, article_query))
    }
}

fn parse_query(
    query: &str,
    article_filter: &mut Option<&mut ArticleFilter>,
) -> Result<ArticleQuery, color_eyre::Report> {
    let mut article_query = ArticleQuery {
        query_string: query.to_string(),
        ..Default::default()
    };

    let mut query_lexer = QueryToken::lexer(query);
    let mut negate = false;

    let to_error = |msg: &str, slice: &str, pos: usize| -> color_eyre::eyre::Report {
        color_eyre::Report::msg(format!("Invalid query: {msg} but got {} at {}", slice, pos,))
    };

    while let Some(token_result) = query_lexer.next() {
        let token = token_result.map_err(|_| {
            to_error(
                "expected negation or key",
                query_lexer.slice(),
                query_lexer.span().start,
            )
        })?;

        use QueryToken::*;

        // check for negation
        if token == Negate {
            if negate {
                return Err(to_error(
                    "expected key after negation but got {} at {}",
                    query_lexer.slice(),
                    query_lexer.span().start,
                ));
            } else {
                negate = true;
                continue;
            }
        }

        if let Some(query_atom) = match token {
            KeyRead => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.unread = Some(if negate { Read::Unread } else { Read::Read });
                    None
                }
                None => Some(QueryAtom::Read(Read::Read)),
            },
            KeyUnread => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.unread = Some(if negate { Read::Read } else { Read::Unread });
                    None
                }
                None => Some(QueryAtom::Read(Read::Unread)),
            },
            KeyMarked => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.marked = Some(if negate {
                        Marked::Unmarked
                    } else {
                        Marked::Marked
                    });
                    None
                }

                None => Some(QueryAtom::Marked(Marked::Marked)),
            },
            KeyUnmarked => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.marked = Some(if negate {
                        Marked::Marked
                    } else {
                        Marked::Unmarked
                    });
                    None
                }
                None => Some(QueryAtom::Marked(Marked::Unmarked)),
            },

            key @ (KeyTitle | KeySummary | KeyAuthor | KeyFeed | KeyFeedUrl | KeyFeedWebUrl
            | KeyAll) => match query_lexer.next() {
                Some(Ok(search_term)) => {
                    let search_term = to_search_term(search_term, query_lexer.slice())?;
                    Some(match key {
                        KeyTitle => QueryAtom::Title(search_term),
                        KeySummary => QueryAtom::Summary(search_term),
                        KeyAuthor => QueryAtom::Author(search_term),
                        KeyFeed => QueryAtom::Feed(search_term),
                        KeyFeedUrl => QueryAtom::FeedUrl(search_term),
                        KeyFeedWebUrl => QueryAtom::FeedWebUrl(search_term),
                        KeyAll => QueryAtom::All(search_term),
                        _ => unreachable!(),
                    })
                }
                _ => {
                    return Err(to_error(
                        "expected regular expression or quoted string",
                        query_lexer.slice(),
                        query_lexer.span().start,
                    ));
                }
            },

            KeyTag => match query_lexer.next() {
                Some(Ok(TagList)) => {
                    let tag_list: Vec<String> = query_lexer
                        .slice()
                        .split(",")
                        .map(&str::to_string)
                        .map(|mut tag| {
                            tag.remove(0);
                            tag
                        })
                        .collect();

                    Some(QueryAtom::Tag(tag_list))
                }
                _ => {
                    return Err(to_error(
                        "expected regular expression or quoted string",
                        query_lexer.slice(),
                        query_lexer.span().start,
                    ));
                }
            },

            mut time_key @ (KeyNewer | KeyOlder | KeySyncedBefore | KeySyncedAfter) => {
                let time = match query_lexer.next() {
                    Some(Ok(QuotedString)) => {
                        let mut time_string = query_lexer.slice().to_string();
                        strip_first_and_last(&mut time_string);
                        let zoned = parse_datetime(&time_string).map_err(|_| {
                            to_error(
                                "expected time string or relative time",
                                query_lexer.slice(),
                                query_lexer.span().start,
                            )
                        })?;
                        DateTime::from_timestamp(
                            zoned.timestamp().as_second(),
                            zoned.timestamp().subsec_nanosecond() as u32,
                        )
                        .unwrap()
                    }

                    _ => {
                        return Err(to_error(
                            "expected time string or relative time",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                };

                if negate {
                    time_key = match time_key {
                        QueryToken::KeyNewer => QueryToken::KeyOlder,
                        QueryToken::KeyOlder => QueryToken::KeyNewer,
                        QueryToken::KeySyncedBefore => QueryToken::KeySyncedAfter,
                        QueryToken::KeySyncedAfter => QueryToken::KeySyncedBefore,
                        _ => unreachable!(),
                    };
                }

                match time_key {
                    QueryToken::KeyNewer => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.newer_than = match article_filter.newer_than {
                                Some(other_time) => Some(other_time.max(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::Newer(time)),
                    },

                    QueryToken::KeyOlder => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.older_than = match article_filter.older_than {
                                Some(other_time) => Some(other_time.min(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::Older(time)),
                    },
                    QueryToken::KeySyncedBefore => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.synced_before = match article_filter.synced_before {
                                Some(other_time) => Some(other_time.min(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::SyncedBefore(time)),
                    },

                    QueryToken::KeySyncedAfter => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.synced_after = match article_filter.synced_after {
                                Some(other_time) => Some(other_time.max(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::SyncedAfter(time)),
                    },

                    _ => unreachable!(),
                }
            }

            QueryToken::Word => Some(QueryAtom::All(SearchTerm::Word(
                query_lexer.slice().to_string(),
            ))),

            _ => {
                return Err(to_error(
                    "expected key but got",
                    query_lexer.source(),
                    query_lexer.span().start,
                ));
            }
        } {
            article_query.query.push(if negate {
                QueryClause::Not(query_atom)
            } else {
                QueryClause::Id(query_atom)
            });
        }

        // reset negate flag
        negate = false;
    }

    trace!("query parsed: {:?}", article_query);

    Ok(article_query)
}

fn strip_first_and_last(s: &mut String) {
    s.remove(0);
    s.remove(s.len() - 1);
}

fn to_search_term(query_token: QueryToken, slice: &str) -> color_eyre::Result<SearchTerm> {
    match query_token {
        QueryToken::Regex => {
            let mut search_term = slice.to_string();
            strip_first_and_last(&mut search_term);

            let regex = regex::Regex::new(&search_term);

            match regex {
                Ok(regex) => Ok(SearchTerm::Regex(regex)),
                Err(err) => Err(color_eyre::Report::msg(format!(
                    "invalid regular expression: {err}"
                ))),
            }
        }

        QueryToken::QuotedString => {
            let mut search_term = slice.to_string();
            strip_first_and_last(&mut search_term);
            Ok(SearchTerm::Verbatim(search_term))
        }

        QueryToken::Word => {
            let search_term = slice.to_string();
            Ok(SearchTerm::Word(search_term))
        }

        _ => Err(color_eyre::Report::msg(
            "expected regular expression or quoted string",
        )),
    }
}
