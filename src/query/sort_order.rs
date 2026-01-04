// use crate::prelude::*;

use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use logos::Logos;
use news_flash::models::{Article, Feed, FeedID};
use serde::Deserialize;

#[derive(Clone, Debug, logos::Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum SortToken {
    #[token(">")]
    Descending,

    #[token("<")]
    Ascending,

    #[token("feed")]
    KeyFeed,

    #[token("date")]
    KeyDate,

    #[token("synced")]
    KeySynced,

    #[token("title")]
    KeyTitle,

    #[token("author")]
    KeyAuthor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn reversed(self) -> Self {
        use SortDirection as S;
        match self {
            S::Ascending => S::Descending,
            S::Descending => S::Ascending,
        }
    }

    fn apply(&self, ordering: Ordering) -> Ordering {
        if matches!(self, SortDirection::Descending) {
            ordering.reverse()
        } else {
            ordering
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    Feed(SortDirection),
    Date(SortDirection),
    Synced(SortDirection),
    Title(SortDirection),
    Author(SortDirection),
}

impl SortKey {
    pub fn reversed(self) -> Self {
        use SortKey as K;
        match self {
            K::Feed(direction) => K::Feed(direction.reversed()),
            K::Date(direction) => K::Date(direction.reversed()),
            K::Synced(direction) => K::Synced(direction.reversed()),
            K::Title(direction) => K::Title(direction.reversed()),
            K::Author(direction) => K::Author(direction.reversed()),
        }
    }

    pub fn compare(
        &self,
        article_1: &Article,
        article_2: &Article,
        feed_map: &HashMap<FeedID, Feed>,
    ) -> Ordering {
        use SortKey as K;

        match self {
            K::Date(direction) => direction
                .apply(article_1.date.cmp(&article_2.date))
                .reverse(),
            K::Title(direction) => direction.apply(
                article_1
                    .title
                    .as_ref()
                    .map(|title| title.to_uppercase())
                    .cmp(&article_2.title.as_ref().map(|title| title.to_uppercase())),
            ),
            K::Synced(direction) => direction
                .apply(article_1.synced.cmp(&article_2.synced))
                .reverse(),
            K::Author(direction) => direction.apply(
                article_1
                    .author
                    .as_ref()
                    .map(|author| author.to_uppercase())
                    .cmp(
                        &article_2
                            .author
                            .as_ref()
                            .map(|author| author.to_uppercase()),
                    ),
            ),
            K::Feed(direction) => {
                let label1 = feed_map
                    .get(&article_1.feed_id)
                    .map(|feed| feed.label.to_uppercase());
                let label2 = feed_map
                    .get(&article_2.feed_id)
                    .map(|feed| feed.label.to_uppercase());
                direction.apply(label1.cmp(&label2))
            }
        }
    }
}

#[derive(Default, Clone, Debug, getset::Getters, Eq, PartialEq)]
pub struct SortOrder {
    #[get = "pub"]
    order: Vec<SortKey>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Default)]
pub enum SortOrderParseError {
    #[default]
    #[error("unknown error")]
    UnknownError,

    #[error("order direction (< or >) or key expected")]
    OrderDirectionOrKeyExpected(usize, String),

    #[error("expecting order key (date, feed, etc.)")]
    OrderKeyExpected(usize, String),

    #[error("duplicate key found")]
    DuplicateKeyFound(usize, String),
}

impl SortOrder {
    #[allow(dead_code)]
    pub fn reversed(self) -> Self {
        SortOrder {
            order: self
                .order
                .into_iter()
                .rev()
                .map(|key| key.reversed())
                .collect(),
        }
    }

    pub fn sort(&self, articles: &mut [Article], feed_map: &HashMap<FeedID, Feed>) {
        if self.order.is_empty() {
            return;
        }
        articles.sort_by(|article_1, article_2| {
            for key in self.order.iter() {
                let ordering = key.compare(article_1, article_2, feed_map);

                if !matches!(ordering, Ordering::Equal) {
                    return ordering;
                }
            }

            Ordering::Equal
        });
    }
}

impl FromStr for SortOrder {
    type Err = SortOrderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_sort_order(s)
    }
}

impl<'de> Deserialize<'de> for SortOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        SortOrder::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

pub fn parse_sort_order(sort_order_str: &str) -> Result<SortOrder, SortOrderParseError> {
    let mut order = Vec::new();

    let mut lexer = SortToken::lexer(sort_order_str);

    while let Some(token) = lexer.next() {
        use SortDirection as D;
        use SortKey as K;
        use SortToken as T;

        let (direction, next) = match token {
            Ok(T::Ascending) => (D::Ascending, lexer.next()),
            Ok(T::Descending) => (D::Descending, lexer.next()),
            next @ Ok(_) => (D::Ascending, Some(next)), // default is ascending

            _ => {
                return Err(SortOrderParseError::OrderDirectionOrKeyExpected(
                    lexer.span().start,
                    lexer.slice().to_owned(),
                ));
            }
        };

        let key = match next {
            Some(Ok(T::KeyFeed)) => K::Feed(direction),
            Some(Ok(T::KeyDate)) => K::Date(direction),
            Some(Ok(T::KeySynced)) => K::Synced(direction),
            Some(Ok(T::KeyTitle)) => K::Title(direction),
            Some(Ok(T::KeyAuthor)) => K::Author(direction),
            _ => {
                return Err(SortOrderParseError::OrderKeyExpected(
                    lexer.span().start,
                    lexer.slice().to_owned(),
                ));
            }
        };

        if order
            .iter()
            .any(|&order_key| order_key == key || order_key == key.reversed())
        {
            return Err(SortOrderParseError::DuplicateKeyFound(
                lexer.span().start,
                lexer.slice().to_owned(),
            ));
        }

        order.push(key);
    }

    Ok(SortOrder { order })
}

#[cfg(test)]
mod test {

    use super::*;
    use SortDirection as D;
    use SortKey as K;
    use rstest::rstest;

    #[rstest]
    #[case("", vec![])]
    #[case("date", vec![K::Date(D::Ascending)])]
    #[case(">date", vec![K::Date(D::Descending)])]
    #[case("<date", vec![K::Date(D::Ascending)])]
    #[case(">feed", vec![K::Feed(D::Descending)])]
    #[case(">author", vec![K::Author(D::Descending)])]
    #[case(">synced", vec![K::Synced(D::Descending)])]
    #[case(">title", vec![K::Title(D::Descending)])]
    #[case("   <title", vec![K::Title(D::Ascending)])]
    #[case("<     date", vec![K::Date(D::Ascending)])]
    #[case(">date      ", vec![K::Date(D::Descending)])]
    #[case("<feed >date", vec![K::Feed(D::Ascending), K::Date(D::Descending)])]
    #[case("<feed     >date author", vec![K::Feed(D::Ascending), K::Date(D::Descending), K::Author(D::Ascending)])]
    #[case("<feed     >date <author >title", vec![K::Feed(D::Ascending), K::Date(D::Descending), K::Author(D::Ascending), K::Title(D::Descending)])]
    fn test_parse_sort_order(#[case] sort_order: &str, #[case] parsed: Vec<SortKey>) {
        assert_eq!(parse_sort_order(sort_order).unwrap().order, parsed);
    }

    #[rstest]
    #[case("foo")]
    #[case("   foo title")]
    #[case(">title article")]
    #[case(" author feed <date synched")]
    fn test_parse_fail_lexer_error(#[case] sort_order: &str) {
        claims::assert_matches!(
            parse_sort_order(sort_order),
            Err(SortOrderParseError::OrderDirectionOrKeyExpected(..))
        )
    }

    #[rstest]
    #[case("foo")]
    #[case("   foo title")]
    #[case(">title article")]
    #[case(" author feed <date synched")]
    fn test_parse_fail_order_direction_or_key_expected(#[case] sort_order: &str) {
        claims::assert_matches!(
            parse_sort_order(sort_order),
            Err(SortOrderParseError::OrderDirectionOrKeyExpected(..))
        )
    }

    #[rstest]
    #[case("<")]
    #[case("<<")]
    #[case("<>")]
    #[case(" <>")]
    #[case("<foo")]
    #[case("<title author <foo")]
    fn test_parse_fail_order_key_expected(#[case] sort_order: &str) {
        claims::assert_matches!(
            parse_sort_order(sort_order),
            Err(SortOrderParseError::OrderKeyExpected(..))
        )
    }

    #[rstest]
    #[case("title title")]
    #[case("<title title")]
    #[case("<title >title")]
    #[case("title >author <title")]
    #[case("date feed author date")]
    fn test_parse_fail_duplicate_key(#[case] sort_order: &str) {
        claims::assert_matches!(
            parse_sort_order(sort_order),
            Err(SortOrderParseError::DuplicateKeyFound(..))
        )
    }

    #[rstest]
    #[case(parse_sort_order("").unwrap(), parse_sort_order("").unwrap())]
    #[case(parse_sort_order(">date >title").unwrap(), parse_sort_order("<title <date").unwrap())]
    #[case(parse_sort_order("<date >title").unwrap(), parse_sort_order("<title >date").unwrap())]
    #[case(parse_sort_order("<date >title <author").unwrap(), parse_sort_order(">author <title >date").unwrap())]
    fn test_sort_order_reversed(
        #[case] sort_order: SortOrder,
        #[case] reversed_sort_order: SortOrder,
    ) {
        assert_eq!(sort_order.reversed(), reversed_sort_order);
    }
}
