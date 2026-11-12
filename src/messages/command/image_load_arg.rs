// use std::str::FromStr;

use std::{fmt::Display, str::FromStr};

use logos::Logos;
use serde::Deserialize;
use strum::EnumMessage;

use crate::prelude::*;

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum ImageLoadToken {
    #[token("visible")]
    VisibleToken,

    #[token("hint")]
    HintToken,

    #[regex(r#"[a-z0-9]+"#)]
    Hint,
}

#[derive(
    Default, Debug, Clone, strum::EnumIter, strum::EnumMessage, strum::AsRefStr, PartialEq, Eq,
)]
pub enum ImageLoadArg {
    #[default]
    #[strum(
        serialize = "visible",
        message = "visible",
        detailed_message = "visible images"
    )]
    Visible,
    #[strum(serialize = "hint", message = "hint", detailed_message = "URL hint")]
    Hint(String),
}

impl<'de> Deserialize<'de> for ImageLoadArg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        ImageLoadArg::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Display for ImageLoadArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

impl FromStr for ImageLoadArg {
    type Err = CommandParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = ImageLoadToken::lexer(s);

        use ImageLoadToken as T;

        Ok(match lexer.next() {
            Some(Ok(T::VisibleToken)) => ImageLoadArg::Visible,
            Some(Ok(T::HintToken)) => match lexer.next() {
                Some(Ok(T::Hint)) => ImageLoadArg::Hint(lexer.slice().to_owned()),
                _ => {
                    return Err(CommandParseError::LoadImageArgExpected(
                        "expected URL hint".to_string(),
                    ));
                }
            },
            _ => {
                return Err(CommandParseError::LoadImageArgExpected(
                    "nothing to parse".to_string(),
                ));
            }
        })
    }
}
