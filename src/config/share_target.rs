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
        Ok(match s.trim() {
            "clipboard" => T::Clipboard,
            "instapaper" => T::Instapaper,
            "reddit" => T::Reddit,
            "mastodon" => T::Mastodon,
            "telegram" => T::Telegram,
            s => match s.split_once(' ') {
                Some((mut name, mut rest)) => {
                    name = name.trim();
                    rest = rest.trim();
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
    fn to_url(&self, title: &str, url: &Url) -> Result<Url, ConfigError> {
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

            ShareTarget::Command(_, args) => Self::execute_as_command(args, title, url),

            _ => {
                let share_url = self.to_url(title, url)?;
                webbrowser::open(share_url.to_string().as_str())?;
                Ok(())
            }
        }
    }

    fn to_command(
        command_args: &[String],
        title: &str,
        url: &Url,
    ) -> color_eyre::Result<(Command, Vec<String>)> {
        let Some((cmd, args)) = command_args.split_first() else {
            return Err(color_eyre::eyre::eyre!("command is empty"));
        };

        let filled_in_args = args
            .iter()
            .map(|arg| {
                arg.replace("{url}", url.as_ref())
                    .replace("{title}", title.as_ref())
            })
            .collect::<Vec<String>>();

        info!("executing command {} {:?}", cmd, filled_in_args);
        Ok((Command::new(cmd), filled_in_args))
    }

    fn execute_as_command(
        command_args: &[String],
        title: &str,
        url: &Url,
    ) -> color_eyre::Result<()> {
        let (mut cmd, filled_in_args) = Self::to_command(command_args, title, url)?;

        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(filled_in_args)
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

#[cfg(test)]
mod tests {

    use claims::assert_matches;

    use super::*;

    macro_rules! custom_parse_test {
        ($id:expr, $url:expr) => {{
            use ShareTarget as S;
            let custom = ShareTarget::from_str(concat!($id, " ", $url));
            assert_matches!(custom, Ok(S::Custom(..)));
            if let Ok(S::Custom(id, url)) = custom {
                assert_eq!(url, $url.trim());
                assert_eq!(id, $id.trim());
            }
        }};
    }

    macro_rules! command_parse_test {
        ($command:expr, $id:expr, $args:expr) => {{
            use ShareTarget as S;
            let custom = ShareTarget::from_str($command);
            assert_matches!(custom, Ok(S::Command(..)));

            if let Ok(S::Command(id, args)) = custom {
                assert_eq!(args, $args);
                assert_eq!(id, $id.trim());
            }
        }};
    }

    macro_rules! command_replace_test {
        ($inargs:expr, $url:expr, $title:expr, $replaced:expr) => {{
            let args = $inargs
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>();

            let (_command, replaced_args) =
                ShareTarget::to_command(&args, $title, &Url::from_str($url).unwrap()).unwrap();

            assert_eq!(replaced_args, $replaced,);
        }};
    }

    #[test]
    fn test_parsing_predefined() {
        use ShareTarget as S;
        assert_matches!(ShareTarget::from_str("clipboard"), Ok(S::Clipboard));
        assert_matches!(ShareTarget::from_str("instapaper"), Ok(S::Instapaper));
        assert_matches!(ShareTarget::from_str("mastodon"), Ok(S::Mastodon));
        assert_matches!(ShareTarget::from_str("reddit"), Ok(S::Reddit));
        assert_matches!(ShareTarget::from_str("telegram"), Ok(S::Telegram));
        assert_matches!(ShareTarget::from_str("signal"), Err(_));
        assert_matches!(ShareTarget::from_str(""), Err(_));
        assert_matches!(ShareTarget::from_str(" telegram"), Ok(S::Telegram));
    }

    #[test]
    fn test_parsing_custom() {
        custom_parse_test!("shareme", "http://www.shareme.com/{url}/{title}");
        custom_parse_test!(" shareme ", "   http://www.shareme.com/{url}/{title} ");
        custom_parse_test!("shareme ", "https://www.shareme.com/{url}/{title}");
        custom_parse_test!(" shareme ", "https://www.shareme.com/{url}/{title} ");
    }

    #[test]
    fn test_parsing_command() {
        command_parse_test!(r#"fancyprog "a" "b" 'c'"#, "fancyprog", vec!["a", "b", "c"]);
        command_parse_test!(
            r#"    fancyprog     "a"    "b"    'c'    "#,
            "fancyprog",
            vec!["a", "b", "c"]
        );
        command_parse_test!(
            r#"fancyprog "{url}" '{title}'"#,
            "fancyprog",
            vec!["{url}", "{title}"]
        );
        command_parse_test!(
            r#"fancyprog "\"{url}\"" \'{title}\'"#,
            "fancyprog",
            vec!["\"{url}\"", "'{title}'"]
        );

        assert_matches!(
            ShareTarget::from_str(r#"fancyprog "a"#),
            Err(ConfigError::ShareTargetInvalidCommand(..))
        );

        assert_matches!(
            ShareTarget::from_str(r#"fancyprog 'a"#),
            Err(ConfigError::ShareTargetInvalidCommand(..))
        );
    }

    #[test]
    fn test_to_url() {
        // pub fn to_url(&self, title: &str, url: &Url) -> Result<Url, ConfigError> {
        use ShareTarget as T;
        let url = T::Mastodon.to_url(
            "Title",
            &Url::from_str("http://www.newssite.com/article?id=123").unwrap(),
        );
        assert_matches!(url, Ok(Url { .. }));

        assert_eq!(
            url.unwrap(),
            Url::from_str("https://sharetomastodon.github.io/?title=Title&url=http%3A%2F%2Fwww%2Enewssite%2Ecom%2Farticle%3Fid%3D123")
            .unwrap()
        )
    }

    #[test]
    fn test_to_command() {
        command_replace_test!(
            vec!["fancyprog", "-v", "{url}", "{title}"],
            "https://www.newssite.com/article?id=123",
            "Title",
            vec!["-v", "https://www.newssite.com/article?id=123", "Title",]
        );
        command_replace_test!(
            vec!["fancyprog", "-v", "\'{url}\'", "\"{title}\""],
            "https://www.newssite.com/article?id=123",
            "Title",
            vec![
                "-v",
                "\'https://www.newssite.com/article?id=123\'",
                "\"Title\"",
            ]
        );
        command_replace_test!(
            vec!["fancyprog", "-v", "{url}  ", " ? {title} *"],
            "https://www.newssite.com/article?id=123",
            "Title",
            vec![
                "-v",
                "https://www.newssite.com/article?id=123  ",
                " ? Title *",
            ]
        );
    }
}
