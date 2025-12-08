use std::{fmt::Display, str::FromStr};

use crate::prelude::*;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use strum::EnumMessage;
use url::Url;

const REDDIT_URL_TEMPLATE: &str = "http://www.reddit.com/submit?url={url}&title={title}";
const INSTAPAPER_URL_TEMPLATE: &str = "http://www.instapaper.com/hello2?url={url}&title={title}";
const MASTODON_URL_TEMPLATE: &str = "https://sharetomastodon.github.io/?title={title}&url={url}";
const TELEGRAM_URL_TEMPLATE: &str = "tg://msg_url?url={url}&text={title}";

#[derive(Debug, Default, Clone, strum::EnumIter, strum::EnumMessage, strum::IntoStaticStr)]
pub enum ShareTarget {
    #[default]
    #[strum(
        serialize = "clipboard",
        message = "Clipboard",
        detailed_message = "copy URL to clipboard"
    )]
    Clipboard,

    #[strum(
        serialize = "instapaper",
        message = "Instapaper",
        detailed_message = "'read later' web serive and app"
    )]
    Instapaper,
    #[strum(
        serialize = "reddit",
        message = "Reddit",
        detailed_message = "share on Reddit"
    )]
    Reddit,
    #[strum(
        serialize = "mastodon",
        message = "Mastodon",
        detailed_message = "share as Toot on Mastodon"
    )]
    Mastodon,
    #[strum(
        serialize = "telegram",
        message = "Telegram",
        detailed_message = "share on Telegram"
    )]
    Telegram,
    #[strum(message = "Custom", detailed_message = "custom target")]
    Custom(String, String),
}

impl AsRef<str> for ShareTarget {
    fn as_ref(&self) -> &str {
        match self {
            ShareTarget::Custom(name, ..) => name,
            target => target.into(),
        }
    }
}

impl Display for ShareTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ShareTarget as T;
        match self {
            T::Custom(name, ..) => f.write_str(name),
            target => f.write_str(target.get_message().unwrap_or(self.as_ref())),
        }
    }
}

impl FromStr for ShareTarget {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ShareTarget as T;
        Ok(match s {
            "clipboard" => T::Clipboard,
            "instapaper" => T::Instapaper,
            "reddit" => T::Reddit,
            "mastodon" => T::Mastodon,
            "telegram" => T::Telegram,
            s => match s.split_once(' ') {
                Some((name, url_template)) => T::Custom(name.to_owned(), url_template.to_owned()),
                None => {
                    return Err(ConfigError::ShareTargetParseError(format!(
                        "unable to parse share target: {s}"
                    )));
                }
            },
        })
    }
}

impl ShareTarget {
    pub fn to_url(&self, title: &str, url: &Url) -> Result<Url, ConfigError> {
        let url_escaped = utf8_percent_encode(url.as_ref(), NON_ALPHANUMERIC).to_string();
        let title_escaped = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();

        use ShareTarget as T;
        let url_template = match self {
            T::Reddit => REDDIT_URL_TEMPLATE,
            T::Instapaper => INSTAPAPER_URL_TEMPLATE,
            T::Mastodon => MASTODON_URL_TEMPLATE,
            T::Telegram => TELEGRAM_URL_TEMPLATE,
            T::Custom(_, url_template) => url_template,
            T::Clipboard => return Err(ConfigError::ShareTargetInvalid),
        };

        let url_str = url_template
            .to_owned()
            .replace("{url}", &url_escaped)
            .replace("{title}", &title_escaped);
        Ok(Url::parse(&url_str)?)
    }
}

impl<'de> serde::de::Deserialize<'de> for ShareTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        ShareTarget::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
