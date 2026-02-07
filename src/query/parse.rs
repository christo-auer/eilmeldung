use std::str::FromStr;

use crate::{
    prelude::*,
    query::{QueryAtom, QueryClause, SearchTerm},
};

use chrono::DateTime;
use log::trace;
use logos::Logos;
use news_flash::models::{ArticleFilter, Marked, Read};
use parse_datetime::parse_datetime;

impl FromStr for ArticleQuery {
    type Err = QueryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_query(s, &mut None)
    }
}

impl FromStr for AugmentedArticleFilter {
    type Err = QueryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut article_filter = ArticleFilter::default();
        let article_query = parse_query(s, &mut Some(&mut article_filter))?;
        Ok(AugmentedArticleFilter::new(article_filter, article_query))
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

    #[error("expecting sort order (e.g., \"date <feed\")")]
    SortOrderExpected(usize, String),

    #[error("multiple sort orders found, only one sort order allowed")]
    MultipleSortOrdersFound(usize, String),

    #[error("expecting time or relative time")]
    TimeOrRelativeTimeExpected(usize, String),

    #[error("invalid regular expression")]
    InvalidRegularExpression(#[from] regex::Error),

    #[error("invalid sort order")]
    InvalidSortOrder(#[from] SortOrderParseError),
}

impl QueryParseError {
    fn from_lexer(lexer: &mut logos::Lexer<'_, QueryToken>) -> Self {
        QueryParseError::LexerError(lexer.span().start, lexer.slice().to_owned())
    }
}

#[derive(Logos, Debug, PartialEq, strum::EnumIter, strum::EnumMessage, strum::AsRefStr)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error(QueryParseError, QueryParseError::from_lexer))]
pub enum QueryToken {
    #[token("~", priority = 2)]
    #[strum(serialize = "~", message = "~", detailed_message = "negation ('not')")]
    Negate,

    #[token("*", priority = 2)]
    #[strum(serialize = "*", message = "*", detailed_message = "matches all")]
    KeyTrue,

    #[token("read", priority = 2)]
    #[strum(
        serialize = "read",
        message = "read",
        detailed_message = "read articles"
    )]
    KeyRead,

    #[token("unread", priority = 2)]
    #[strum(
        serialize = "unread",
        message = "unread",
        detailed_message = "unread articles"
    )]
    KeyUnread,

    #[token("marked", priority = 2)]
    #[strum(
        serialize = "marked",
        message = "marked",
        detailed_message = "marked articles"
    )]
    KeyMarked,

    #[token("unmarked", priority = 2)]
    #[strum(
        serialize = "unmarked",
        message = "unmarked",
        detailed_message = "unmarked articles"
    )]
    KeyUnmarked,

    #[token("tagged", priority = 2)]
    #[strum(
        serialize = "tagged",
        message = "tagged",
        detailed_message = "articles with a tag"
    )]
    KeyTagged,

    #[token("newer:")]
    #[strum(
        serialize = "newer",
        message = "newer:<time>",
        detailed_message = "articles newer than the defined time"
    )]
    KeyNewer,

    #[token("older:")]
    #[strum(
        serialize = "older",
        message = "older:<time>",
        detailed_message = "articles older than the defined time"
    )]
    KeyOlder,

    #[token("today")]
    #[strum(
        serialize = "today",
        message = "today",
        detailed_message = "articles from today"
    )]
    KeyToday,

    #[token("lastsync")]
    #[strum(
        serialize = "lastsync",
        message = "lastsync",
        detailed_message = "articles retrieved in the last sync operation"
    )]
    KeyLastSync,

    #[token("syncedbefore:")]
    #[strum(
        serialize = "syncedbefore",
        message = "syncedbefore:<time>",
        detailed_message = "articles synced before the defined time"
    )]
    KeySyncedBefore,

    #[token("syncedafter:")]
    #[strum(
        serialize = "syncedafter",
        message = "syncedafter:<time>",
        detailed_message = "articles synced after the defined time"
    )]
    KeySyncedAfter,

    #[token("feed:")]
    #[strum(
        serialize = "feedafter",
        message = "feed:<search term>",
        detailed_message = "articles with a feed matching the search term"
    )]
    KeyFeed,

    #[token("title:")]
    #[strum(
        serialize = "title",
        message = "title:<search term>",
        detailed_message = "articles with a title matching the search term"
    )]
    KeyTitle,

    #[token("summary:")]
    #[strum(
        serialize = "summary",
        message = "summary:<search term>",
        detailed_message = "articles with a summary matching the search term"
    )]
    KeySummary,

    #[token("author:")]
    #[strum(
        serialize = "author",
        message = "author:<search term>",
        detailed_message = "articles with an author matching the search term"
    )]
    KeyAuthor,

    #[token("all:")]
    #[strum(
        serialize = "all",
        message = "all:<search term>",
        detailed_message = "articles with any field containing the search term"
    )]
    KeyAll,

    #[token("feedurl:")]
    #[strum(
        serialize = "feedurl",
        message = "feedurl:<search term>",
        detailed_message = "articles with a feed URL containing the search term"
    )]
    KeyFeedUrl,

    #[token("feedweburl:")]
    #[strum(
        serialize = "feedweburl",
        message = "feedweburl:<search term>",
        detailed_message = "articles with a feed web URL containing the search term"
    )]
    KeyFeedWebUrl,

    #[token("tag:")]
    #[strum(
        serialize = "tag",
        message = "tag:<tag list>",
        detailed_message = "articles containing all listed tags"
    )]
    KeyTag,

    #[token("sort:")]
    #[strum(
        serialize = "sort",
        message = "tag:\"<sort order>\"",
        detailed_message = "sorts the articles by the given sort order"
    )]
    Sort,

    #[regex(r#""[^"\n\r\\]*(?:\\.[^"\n\r\\]*)*""#)]
    QuotedString,

    #[regex(r#"[\w]+"#, priority = 1)]
    Word,

    #[regex(r"/[^/\\]*(?:\\.[^/\\]*)*/")]
    Regex,

    #[regex(r#"#[a-zA-Z][a-zA-Z0-9]*(?:,#[a-zA-Z][a-zA-Z0-9]*)*"#)]
    TagList,
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
            T::KeyTrue => Some(QueryAtom::True),

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
            T::KeyLastSync => Some(QueryAtom::LastSync),

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

            tag @ (T::KeyTag | T::TagList) => {
                let tag_list_str = match tag {
                    T::KeyTag => match query_lexer.next() {
                        Some(Ok(T::TagList)) => query_lexer.slice(),

                        _ => {
                            return Err(E::TagListExpected(
                                query_lexer.span().start,
                                query_lexer.slice().to_owned(),
                            ));
                        }
                    },
                    T::TagList => query_lexer.slice(),

                    _ => unreachable!(),
                };
                let tag_list: Vec<String> = tag_list_str
                    .split(",")
                    .map(&str::to_string)
                    .map(|mut tag| {
                        tag.remove(0);
                        tag
                    })
                    .collect();

                Some(QueryAtom::Tag(tag_list))
            }

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

            QueryToken::Sort => match query_lexer.next() {
                Some(Ok(T::QuotedString)) => {
                    let mut sort_order = query_lexer.slice().to_owned();
                    strip_first_and_last(&mut sort_order);
                    if article_query.sort_order.is_none() {
                        article_query.sort_order = Some(SortOrder::from_str(&sort_order)?);
                    } else {
                        return Err(QueryParseError::MultipleSortOrdersFound(
                            query_lexer.span().start,
                            query_lexer.slice().to_owned(),
                        ));
                    }
                    None
                }

                _ => {
                    return Err(QueryParseError::SortOrderExpected(
                        query_lexer.span().start,
                        query_lexer.slice().to_owned(),
                    ));
                }
            },

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
