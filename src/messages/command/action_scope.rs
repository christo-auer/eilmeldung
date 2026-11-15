use std::{fmt::Display, str::FromStr};

use crate::prelude::*;

#[derive(Clone, Debug, Default, strum::EnumIter, strum::EnumMessage, strum::AsRefStr)]
pub enum ActionScope {
    #[default]
    #[strum(
        serialize = "current",
        message = "current",
        detailed_message = "currently selected item"
    )]
    Current,

    #[strum(
        serialize = "above",
        message = "above",
        detailed_message = "currently selected item and all above"
    )]
    Above,

    #[strum(
        serialize = "below",
        message = "below",
        detailed_message = "currently selected item and all below"
    )]
    Below,

    #[strum(serialize = "all", message = "all", detailed_message = "all items")]
    All,
    #[strum(
        message = "query",
        detailed_message = "all articles defined by a query"
    )]
    Query(ArticleQuery),
}

impl FromStr for ActionScope {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ActionScope as S;
        match s {
            "." | "current" => Ok(S::Current),
            "%" | "all" => Ok(S::All),
            "above" => Ok(S::Above),
            "below" => Ok(S::Below),
            _ => Ok(S::Query(ArticleQuery::from_str(s)?)),
        }
    }
}

impl ActionScope {
    pub fn from_option_string(s: Option<&str>) -> Result<ActionScope, CommandParseError> {
        match s {
            Some(s) => Ok(ActionScope::from_str(s)?),
            None => Ok(ActionScope::Current),
        }
    }
}

impl Display for ActionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActionScope as S;
        match self {
            S::Current => write!(f, "current article")?,
            S::All => write!(f, "all articles")?,
            S::Below => write!(f, "current article and all below")?,
            S::Above => write!(f, "current article and all above")?,
            S::Query(query) => write!(f, "all articles matching {}", query.query_string())?,
        };
        Ok(())
    }
}
