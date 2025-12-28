use std::{
    fmt::Display,
    process::{Command, Stdio},
    str::FromStr,
};

use crate::prelude::*;
use arboard::Clipboard;
use log::info;
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

    #[strum(message = "Command", detailed_message = "command")]
    Command(String, Vec<String>),
}

impl AsRef<str> for ShareTarget {
    fn as_ref(&self) -> &str {
        match self {
            ShareTarget::Custom(name, ..) => name,
            ShareTarget::Command(name, ..) => name,
            target => target.into(),
        }
    }
}

impl Display for ShareTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ShareTarget as T;
        match self {
            T::Custom(name, ..) => f.write_str(name),
            T::Command(name, ..) => f.write_str(name),
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
                Some((name, rest)) => {
                    if rest.starts_with("http://") || rest.starts_with("https://") {
                        T::Custom(name.to_owned(), rest.to_owned())
                    } else {
                        T::Command(name.to_owned(), shell_words::split(rest)?)
                    }
                }
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
            T::Command(..) => return Err(ConfigError::ShareTargetInvalid),
        };

        let url_str = url_template
            .to_owned()
            .replace("{url}", &url_escaped)
            .replace("{title}", &title_escaped);
        Ok(Url::parse(&url_str)?)
    }

    pub fn share(&self, title: &str, url: &Url) -> color_eyre::Result<()> {
        match self {
            ShareTarget::Clipboard => {
                let mut clipboard = Clipboard::new()?;
                clipboard.set_text(url.to_string())?;
                Ok(())
            }

            ShareTarget::Command(_, args) => self.execute_as_command(args, title, url),

            target => {
                let share_url = target.to_url(title, url)?;
                webbrowser::open(share_url.to_string().as_str())?;
                Ok(())
            }
        }
    }

    pub fn execute_as_command(
        &self,
        command_args: &[String],
        title: &str,
        url: &Url,
    ) -> color_eyre::Result<()> {
        let Some((cmd, args)) = command_args.split_first() else {
            return Err(color_eyre::eyre::eyre!("command is empty"));
        };

        let quoted_args = args
            .iter()
            .map(|arg| {
                arg.replace("{url}", shell_words::quote(url.as_ref()).as_ref())
                    .replace("{title}", shell_words::quote(title).as_ref())
            })
            .collect::<Vec<String>>();

        info!("executing command {} {:?}", cmd, quoted_args);
        let _child = Command::new(cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(quoted_args)
            .spawn()?;

        Ok(())
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
