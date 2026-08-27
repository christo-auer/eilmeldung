use std::{fmt::Display, str::FromStr};

use crate::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum AppState {
    #[default]
    FeedSelection,
    ArticleSelection,
    ArticleContent,
    ArticleContentDistractionFree,
}

impl From<Panel> for AppState {
    fn from(value: Panel) -> Self {
        match value {
            Panel::FeedList => Self::FeedSelection,
            Panel::ArticleList => Self::ArticleSelection,
            Panel::ArticleContent => Self::ArticleContent,
        }
    }
}

impl FromStr for AppState {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "feeds" => Self::FeedSelection,
            "articles" => Self::ArticleSelection,
            "content" => Self::ArticleContent,
            "zen" => Self::ArticleContentDistractionFree,
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "expected feeds, articles, content or zen"
                ));
            }
        })
    }
}

impl Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppState::FeedSelection => write!(f, "feed selection"),
            AppState::ArticleSelection => write!(f, "article selection"),
            AppState::ArticleContent => write!(f, "article content"),
            AppState::ArticleContentDistractionFree => {
                write!(f, "article content distraction free")
            }
        }
    }
}

impl AppState {
    pub fn previous_cyclic(&self) -> AppState {
        use AppState::*;
        match self {
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            FeedSelection => ArticleContent,
            _ => *self,
        }
    }

    pub fn next_cyclic(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => FeedSelection,
            _ => *self,
        }
    }

    pub fn next(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => ArticleContent,
            _ => *self,
        }
    }

    pub fn previous(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => FeedSelection,
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            _ => *self,
        }
    }
}
