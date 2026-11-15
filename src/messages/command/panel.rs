use std::fmt::Display;
use strum::EnumMessage;

use crate::prelude::*;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    strum::EnumString,
    strum::EnumMessage,
    strum::EnumIter,
    strum::AsRefStr,
)]
#[strum(
    parse_err_fn = CommandParseError::panel_expected,
    parse_err_ty = CommandParseError)]
pub enum Panel {
    #[default]
    #[strum(serialize = "feeds")]
    #[strum(
        message = "feeds",
        detailed_message = "panel with tree of feeds, categories, tags, etc."
    )]
    FeedList,

    #[strum(serialize = "articles")]
    #[strum(
        message = "article list",
        detailed_message = "panel with the list of articles"
    )]
    ArticleList,

    #[strum(serialize = "content")]
    #[strum(
        message = "article content",
        detailed_message = "content of the selected article"
    )]
    ArticleContent,
}
impl Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

impl CommandParseError {
    fn panel_expected(_: &str) -> CommandParseError {
        CommandParseError::PanelExpected
    }
}

impl From<AppState> for Panel {
    fn from(value: AppState) -> Self {
        match value {
            AppState::FeedSelection => Panel::FeedList,
            AppState::ArticleSelection => Panel::ArticleList,
            AppState::ArticleContent | AppState::ArticleContentDistractionFree => {
                Panel::ArticleContent
            }
        }
    }
}
