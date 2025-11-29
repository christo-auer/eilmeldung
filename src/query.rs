use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use getset::Getters;
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
    Tagged,
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

#[derive(Default, Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct ArticleQuery {
    query_string: String,
    query: Vec<QueryClause>,
}

impl ArticleQuery {
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
    type Err = QueryParseError;

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

#[derive(Debug, thiserror::Error, Clone, PartialEq, Default)]
pub enum QueryParseError {
    #[default]
    #[error("unknown error")]
    UnknownError,

    #[error("invalid token")]
    LexerError(usize, String),

    #[error("expecting key (title:, newer:, ...) or word to search")]
    KeyOrWordExpected(usize, String),

    #[error("expecting key after negation (~key:...)")]
    KeyAfterNegationExpected(usize, String),

    #[error("expecting search term (unquoted word, regex or quoted string)")]
    SearchTermExpected(usize, String),

    #[error("expecting tag list (#tag1,#tag2,#tag3,...)")]
    TagListExpected(usize, String),

    #[error("expecting time or relative time")]
    TimeOrRelativeTimeExpected(usize, String),

    #[error("invalid regular expression")]
    InvalidRegularExpression(#[from] regex::Error),
}

impl QueryParseError {
    fn from_lexer(lexer: &mut logos::Lexer<'_, QueryToken>) -> Self {
        QueryParseError::LexerError(lexer.span().start, lexer.slice().to_owned())
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error(QueryParseError, QueryParseError::from_lexer))]
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

    #[token("tagged", priority = 2)]
    KeyTagged,

    #[token("newer:")]
    KeyNewer,

    #[token("older:")]
    KeyOlder,

    #[token("today")]
    KeyToday,

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

    pub fn defines_scope(&self) -> bool {
        self.is_augmented()
            || self.article_filter.unread.is_some()
            || self.article_filter.marked.is_some()
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
) -> Result<ArticleQuery, QueryParseError> {
    use QueryParseError as E;
    use QueryToken as T;
    let mut article_query = ArticleQuery {
        query_string: query.to_string(),
        ..Default::default()
    };

    let mut query_lexer = QueryToken::lexer(query);
    let mut negate = false;

    while let Some(token_result) = query_lexer.next() {
        let token = token_result?;

        // check for negation
        if token == T::Negate {
            if negate {
                return Err(E::KeyAfterNegationExpected(
                    query_lexer.span().start,
                    query_lexer.slice().to_owned(),
                ));
            } else {
                negate = true;
                continue;
            }
        }

        if let Some(query_atom) = match token {
            T::KeyRead => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.unread = Some(if negate { Read::Unread } else { Read::Read });
                    None
                }
                None => Some(QueryAtom::Read(Read::Read)),
            },
            T::KeyUnread => match article_filter.as_mut() {
                Some(article_filter) => {
                    article_filter.unread = Some(if negate { Read::Read } else { Read::Unread });
                    None
                }
                None => Some(QueryAtom::Read(Read::Unread)),
            },
            T::KeyMarked => match article_filter.as_mut() {
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
            T::KeyUnmarked => match article_filter.as_mut() {
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
            T::KeyTagged => Some(QueryAtom::Tagged),

            key @ (T::KeyTitle
            | T::KeySummary
            | T::KeyAuthor
            | T::KeyFeed
            | T::KeyFeedUrl
            | T::KeyFeedWebUrl
            | T::KeyAll) => match query_lexer.next() {
                Some(Ok(search_term)) => {
                    let search_term = to_search_term(search_term, &query_lexer)?;
                    Some(match key {
                        T::KeyTitle => QueryAtom::Title(search_term),
                        T::KeySummary => QueryAtom::Summary(search_term),
                        T::KeyAuthor => QueryAtom::Author(search_term),
                        T::KeyFeed => QueryAtom::Feed(search_term),
                        T::KeyFeedUrl => QueryAtom::FeedUrl(search_term),
                        T::KeyFeedWebUrl => QueryAtom::FeedWebUrl(search_term),
                        T::KeyAll => QueryAtom::All(search_term),
                        _ => unreachable!(),
                    })
                }
                _ => {
                    return Err(E::SearchTermExpected(
                        query_lexer.span().start,
                        query_lexer.slice().to_owned(),
                    ));
                }
            },

            T::KeyTag => match query_lexer.next() {
                Some(Ok(T::TagList)) => {
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
                    return Err(E::TagListExpected(
                        query_lexer.span().start,
                        query_lexer.slice().to_owned(),
                    ));
                }
            },

            mut time_key @ (T::KeyNewer
            | T::KeyOlder
            | T::KeyToday
            | T::KeySyncedBefore
            | T::KeySyncedAfter) => {
                let time = if matches!(time_key, T::KeyToday) {
                    time_key = T::KeyNewer;

                    let zoned = parse_datetime("1 day ago").unwrap();
                    DateTime::from_timestamp(
                        zoned.timestamp().as_second(),
                        zoned.timestamp().subsec_nanosecond() as u32,
                    )
                    .unwrap()
                } else {
                    match query_lexer.next() {
                        Some(Ok(T::QuotedString)) => {
                            let mut time_string = query_lexer.slice().to_string();
                            strip_first_and_last(&mut time_string);
                            let zoned = parse_datetime(&time_string).map_err(|_| {
                                E::TimeOrRelativeTimeExpected(
                                    query_lexer.span().start,
                                    query_lexer.slice().to_owned(),
                                )
                            })?;
                            DateTime::from_timestamp(
                                zoned.timestamp().as_second(),
                                zoned.timestamp().subsec_nanosecond() as u32,
                            )
                            .unwrap()
                        }

                        _ => {
                            return Err(E::TimeOrRelativeTimeExpected(
                                query_lexer.span().start,
                                query_lexer.slice().to_owned(),
                            ));
                        }
                    }
                };

                if negate {
                    time_key = match time_key {
                        T::KeyNewer => T::KeyOlder,
                        T::KeyOlder => T::KeyNewer,
                        T::KeySyncedBefore => T::KeySyncedAfter,
                        T::KeySyncedAfter => T::KeySyncedBefore,
                        _ => unreachable!(),
                    };
                }

                match time_key {
                    T::KeyNewer => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.newer_than = match article_filter.newer_than {
                                Some(other_time) => Some(other_time.max(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::Newer(time)),
                    },

                    T::KeyOlder => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.older_than = match article_filter.older_than {
                                Some(other_time) => Some(other_time.min(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::Older(time)),
                    },
                    T::KeySyncedBefore => match article_filter.as_mut() {
                        Some(article_filter) => {
                            article_filter.synced_before = match article_filter.synced_before {
                                Some(other_time) => Some(other_time.min(time)),
                                None => Some(time),
                            };
                            None
                        }
                        None => Some(QueryAtom::SyncedBefore(time)),
                    },

                    T::KeySyncedAfter => match article_filter.as_mut() {
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
                return Err(E::KeyOrWordExpected(
                    query_lexer.span().start,
                    query_lexer.slice().to_owned(),
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

fn to_search_term(
    query_token: QueryToken,
    lexer: &logos::Lexer<'_, QueryToken>,
) -> Result<SearchTerm, QueryParseError> {
    match query_token {
        QueryToken::Regex => {
            let mut search_term = lexer.slice().to_owned();
            strip_first_and_last(&mut search_term);

            let regex = regex::Regex::new(&search_term)?;
            Ok(SearchTerm::Regex(regex))
        }

        QueryToken::QuotedString => {
            let mut search_term = lexer.slice().to_owned();
            strip_first_and_last(&mut search_term);
            Ok(SearchTerm::Verbatim(search_term))
        }

        QueryToken::Word => {
            let search_term = lexer.slice().to_owned();
            Ok(SearchTerm::Word(search_term))
        }

        _ => Err(QueryParseError::SearchTermExpected(
            lexer.span().start,
            lexer.slice().to_owned(),
        )),
    }
}
