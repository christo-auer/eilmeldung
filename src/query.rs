use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use chrono::DateTime;
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
    Regex(Regex),
}

#[derive(Clone, Debug)]
enum QueryAtom {
    Feed(SearchTerm),
    Title(SearchTerm),
    Summary(SearchTerm),
    Author(SearchTerm),
    FeedUrl(SearchTerm),
    FeedWebUrl(SearchTerm),
    Tag(Vec<String>),
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
pub struct ParsedQuery {
    query: Vec<QueryClause>,
}

impl QueryAtom {
    #[inline(always)]
    pub fn test(
        &self,
        article: &Article,
        feed: Option<&Feed>,
        tags: Option<&HashSet<String>>,
    ) -> bool {
        match self {
            QueryAtom::Feed(search_term)
            | QueryAtom::Title(search_term)
            | QueryAtom::Summary(search_term)
            | QueryAtom::Author(search_term)
            | QueryAtom::FeedUrl(search_term)
            | QueryAtom::FeedWebUrl(search_term) => {
                self.test_string_match(search_term, article, feed)
            }
            QueryAtom::Tag(search_tags) => {
                let Some(tags) = tags else {
                    return false;
                };
                search_tags.iter().any(|tag| tags.contains(tag))
            }
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
            _ => unreachable!(),
        };

        let Some(content_string) = content_string else {
            return false;
        };

        match search_term {
            SearchTerm::Regex(regex) => regex.is_match(&content_string),
            SearchTerm::Verbatim(term) => content_string.contains(term),
        }
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum QueryToken {
    #[token("~")]
    Negate,

    #[token("read")]
    KeyRead,

    #[token("unread")]
    KeyUnread,

    #[token("marked")]
    KeyMarked,

    #[token("unmarked")]
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

    #[token("feedurl:")]
    KeyFeedUrl,

    #[token("feedweburl:")]
    KeyFeedWebUrl,

    #[token("tag:")]
    KeyTag,

    #[regex(r#""[^"\n\r\\]*(?:\\.[^"\n\r\\]*)*""#)]
    QuotedString,

    #[regex(r"/[^/\\]*(?:\\.[^/\\]*)*/")]
    Regex,

    #[regex(r#"#[a-zA-Z][a-zA-Z0-9]*(?:,#[a-zA-Z][a-zA-Z0-9]*)*"#)]
    TagList,
}

#[derive(Default, Clone, Debug)]
pub struct AugmentedArticleFilter {
    pub article_filter: ArticleFilter,
    pub parsed_query: ParsedQuery,
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
    pub fn new(article_filter: ArticleFilter) -> Self {
        Self {
            article_filter,
            ..Self::default()
        }
    }

    pub fn is_augmented(&self) -> bool {
        !self.parsed_query.query.is_empty()
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
            .filter(|article| {
                let feed = feed_map.get(&article.feed_id);

                let tags = tags_for_article.get(&article.article_id).map(|tag_ids| {
                    tag_ids
                        .iter()
                        .filter_map(|tag_id| tag_map.get(tag_id).map(|tag| tag.label.to_string()))
                        .collect::<HashSet<String>>()
                });

                self.parsed_query
                    .query
                    .iter()
                    .all(|query_clause| query_clause.test(article, feed, tags.as_ref()))
            })
            .cloned()
            .collect::<Vec<Article>>()
    }
}

impl FromStr for AugmentedArticleFilter {
    type Err = color_eyre::Report;

    fn from_str(query: &str) -> Result<Self, Self::Err> {
        let mut filter: AugmentedArticleFilter = Self::default();
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
                KeyRead => {
                    filter.article_filter.unread =
                        Some(if negate { Read::Unread } else { Read::Read });
                    None
                }
                KeyUnread => {
                    filter.article_filter.unread =
                        Some(if negate { Read::Read } else { Read::Unread });
                    None
                }
                KeyMarked => {
                    filter.article_filter.marked = Some(if negate {
                        Marked::Unmarked
                    } else {
                        Marked::Marked
                    });
                    None
                }
                KeyUnmarked => {
                    filter.article_filter.marked = Some(if negate {
                        Marked::Marked
                    } else {
                        Marked::Unmarked
                    });
                    None
                }
                KeyTitle => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::Title(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
                    _ => {
                        return Err(to_error(
                            "expected regular expression or quoted string",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                },
                KeySummary => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::Summary(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
                    _ => {
                        return Err(to_error(
                            "expected regular expression or quoted string",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                },
                KeyAuthor => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::Author(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
                    _ => {
                        return Err(to_error(
                            "expected regular expression or quoted string",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                },
                KeyFeed => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::Feed(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
                    _ => {
                        return Err(to_error(
                            "expected regular expression or quoted string",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                },
                KeyFeedUrl => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::FeedUrl(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
                    _ => {
                        return Err(to_error(
                            "expected regular expression or quoted string",
                            query_lexer.slice(),
                            query_lexer.span().start,
                        ));
                    }
                },
                KeyFeedWebUrl => match query_lexer.next() {
                    Some(Ok(search_term)) => Some(QueryAtom::FeedWebUrl(to_search_term(
                        search_term,
                        query_lexer.slice(),
                    )?)),
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
                        QueryToken::KeyNewer => {
                            filter.article_filter.newer_than =
                                match filter.article_filter.newer_than {
                                    Some(other_time) => Some(other_time.max(time)),
                                    None => Some(time),
                                }
                        }

                        QueryToken::KeyOlder => {
                            filter.article_filter.older_than =
                                match filter.article_filter.older_than {
                                    Some(other_time) => Some(other_time.min(time)),
                                    None => Some(time),
                                }
                        }
                        QueryToken::KeySyncedBefore => {
                            filter.article_filter.synced_before =
                                match filter.article_filter.synced_before {
                                    Some(other_time) => Some(other_time.min(time)),
                                    None => Some(time),
                                }
                        }

                        QueryToken::KeySyncedAfter => {
                            filter.article_filter.synced_after =
                                match filter.article_filter.synced_after {
                                    Some(other_time) => Some(other_time.max(time)),
                                    None => Some(time),
                                }
                        }

                        _ => unreachable!(),
                    }

                    None
                }

                _ => {
                    return Err(to_error(
                        "expected key but got",
                        query_lexer.source(),
                        query_lexer.span().start,
                    ));
                }
            } {
                filter.parsed_query.query.push(if negate {
                    QueryClause::Not(query_atom)
                } else {
                    QueryClause::Id(query_atom)
                });
            }

            // reset negate flag
            negate = false;
        }

        trace!("query parsed: {:?}", filter);

        Ok(filter)
    }
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

        _ => Err(color_eyre::Report::msg(
            "expected regular expression or quoted string",
        )),
    }
}
