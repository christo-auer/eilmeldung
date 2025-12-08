use logos::Logos;
use serde::Deserialize;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum FeedListItemType {
    Tree,
    List,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, serde::Deserialize)]
pub struct LabeledQuery {
    pub label: String,
    pub query: String,
}

impl From<(String, String)> for LabeledQuery {
    fn from((label, query): (String, String)) -> Self {
        Self { label, query }
    }
}

#[derive(Clone, Debug)]
pub enum FeedListContentIdentifier {
    Feeds(FeedListItemType),
    Categories(FeedListItemType),
    Tags(FeedListItemType),
    Query(LabeledQuery),
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum FeedListContentIdentifierToken {
    #[token("*")]
    KeyList,

    #[token("feeds")]
    KeyFeeds,

    #[token("categories")]
    KeyCategories,

    #[token("tags")]
    KeyTags,

    #[token("query:")]
    KeyQuery,

    #[regex(r#""[^"\n\r\\]*(?:\\.[^"\n\r\\]*)*""#)]
    QuotedString,

    #[regex(r#"#[a-zA-Z][a-zA-Z0-9]*"#)]
    Tag,
}

impl FeedListContentIdentifier {
    fn coerce_to_list(self) -> Self {
        use FeedListContentIdentifier::*;
        match self {
            Feeds(_) => Feeds(FeedListItemType::List),
            Tags(_) => Tags(FeedListItemType::List),
            Categories(_) => Categories(FeedListItemType::List),
            other => other,
        }
    }
}

impl FromStr for FeedListContentIdentifier {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = FeedListContentIdentifierToken::lexer(s);

        use FeedListContentIdentifier::*;
        use FeedListContentIdentifierToken::*;
        Ok(match lexer.next() {
            Some(Ok(KeyList)) => {
                let identifier = Self::from_str(lexer.remainder())?;
                identifier.coerce_to_list()
            }
            Some(Ok(KeyFeeds)) => Feeds(FeedListItemType::Tree),
            Some(Ok(KeyCategories)) => Categories(FeedListItemType::Tree),
            Some(Ok(KeyTags)) => Tags(FeedListItemType::Tree),
            Some(Ok(KeyQuery)) => {
                let Some(Ok(QuotedString)) = lexer.next() else {
                    return Err(ConfigError::FeedListContentIdentifierParseError(
                        "expected query label in double quotes".to_owned(),
                    ));
                };
                let label_slice = lexer.slice();
                let label = label_slice[1..label_slice.len() - 1].to_owned();
                let query = lexer.remainder().trim().to_owned();
                Query(LabeledQuery { label, query })
            }
            _ => {
                return Err(ConfigError::FeedListContentIdentifierParseError(format!(
                    "unknown feed list content id: {}",
                    lexer.slice()
                )));
            }
        })
    }
}

impl<'de> Deserialize<'de> for FeedListContentIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = String::deserialize(deserializer)?;

        FeedListContentIdentifier::from_str(&content)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
