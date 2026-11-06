use std::fmt::Display;
use strum::EnumMessage;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    serde::Deserialize,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumMessage,
    strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
pub enum PipeTarget {
    #[strum(serialize = "null", message = "null", detailed_message = "null")]
    #[default]
    Null,

    #[strum(serialize = "html", message = "html", detailed_message = "HTML")]
    Html,

    #[strum(serialize = "md", message = "md", detailed_message = "markdown")]
    Markdown,
}

impl Display for PipeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f)
    }
}
