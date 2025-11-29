pub mod input_config;
pub mod paths;
pub mod theme;

use std::str::FromStr;

use crate::prelude::*;

pub mod prelude {
    pub use super::FeedListContentIdentifier;
    pub use super::FeedListItemType;
    pub use super::input_config::InputConfig;
    pub use super::paths::{CONFIG_FILE, PROJECT_DIRS};
    pub use super::theme::Theme;
    pub use super::{ArticleContentType, ArticleScope, Config, LabeledQuery, load_config};
}

use log::{debug, error, info};
use logos::Logos;
use serde::Deserialize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum ArticleContentType {
    PlainText,
    Markdown,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, serde::Deserialize)]
pub struct LabeledQuery {
    pub label: String,
    pub query: String,
}

#[derive(Debug, Clone)]
pub enum FeedListItemType {
    Tree,
    List,
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
    type Err = color_eyre::Report;
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
                    return Err(color_eyre::eyre::eyre!("expected #tag after tag:"));
                };
                let label = lexer.slice().to_owned();
                let query = lexer.remainder().trim().to_owned();
                Query(LabeledQuery { label, query })
            }
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "unknown feed list content id: {}",
                    lexer.slice()
                ));
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

        Ok(FeedListContentIdentifier::from_str(&content)
            .map_err(|err| serde::de::Error::custom(err.to_string()))?)
    }
}

impl From<(String, String)> for LabeledQuery {
    fn from((label, query): (String, String)) -> Self {
        Self { label, query }
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Default,
    strum::EnumIter,
    strum::EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum ArticleScope {
    #[default]
    All,
    Unread,
    Marked,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub input_config: InputConfig,
    pub theme: Theme,

    pub refresh_fps: u64,

    pub offline_icon: char,
    pub all_label: String,
    pub feed_label: String,
    pub category_label: String,
    pub categories_label: String,
    pub tags_label: String,
    pub tag_label: String,
    pub query_label: String,
    pub tag_icon: char,
    pub article_table: String,
    pub date_format: String,
    pub read_icon: char,
    pub unread_icon: char,
    pub marked_icon: char,
    pub unmarked_icon: char,
    pub command_line_prompt_icon: char,
    pub article_scope: ArticleScope,

    pub articles_list_visible_articles_after_selection: usize,
    pub article_auto_scrape: bool,
    pub article_thumbnail_show: bool,
    pub article_thumbnail_width: u16,
    pub article_thumbnail_resize: bool,
    pub article_thumbnail_fetch_debounce_millis: u64,
    pub article_content_max_chars_per_line: u16,
    pub article_content_preferred_type: ArticleContentType,

    pub feed_list: Vec<FeedListContentIdentifier>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_label: "󱀂 All {unread_count}".into(),
            feed_label: " {label} {unread_count}".into(),
            category_label: "󰉋 {label} {unread_count}".into(),
            categories_label: "󰉓 Categories {unread_count}".into(),
            tags_label: "󰓻 Tags {unread_count}".into(),
            tag_label: "󰓹 {label} {unread_count}".into(),
            query_label: " {label}".into(),
            article_table: "{read},{marked},{tag_icons},{age},{title}".into(),
            date_format: "%m/%d %H:%M".into(),
            theme: Default::default(),
            input_config: Default::default(),
            refresh_fps: 10,
            offline_icon: '',
            read_icon: '',
            unread_icon: '',
            marked_icon: '',
            unmarked_icon: ' ',
            tag_icon: '󰓹',
            command_line_prompt_icon: '',
            article_scope: ArticleScope::Unread,

            articles_list_visible_articles_after_selection: 3,
            article_auto_scrape: true,
            article_thumbnail_show: true,
            article_thumbnail_width: 20,
            article_thumbnail_resize: true,
            article_thumbnail_fetch_debounce_millis: 500,
            article_content_max_chars_per_line: 66,
            article_content_preferred_type: ArticleContentType::Markdown,

            feed_list: vec![
                FeedListContentIdentifier::Query(LabeledQuery {
                    label: "Today Unread".to_owned(),
                    query: "today unread".to_owned(),
                }),
                FeedListContentIdentifier::Query(LabeledQuery {
                    label: "Today Marked".to_owned(),
                    query: "today marked".to_owned(),
                }),
                FeedListContentIdentifier::Feeds(FeedListItemType::Tree),
                FeedListContentIdentifier::Categories(FeedListItemType::List),
                FeedListContentIdentifier::Tags(FeedListItemType::Tree),
            ],
        }
    }
}

pub fn load_config() -> color_eyre::Result<Config> {
    let config_path = PROJECT_DIRS
        .config_dir()
        .join(CONFIG_FILE)
        .to_str()
        .unwrap()
        .to_string();

    info!("Loading config from {}", config_path);
    debug!("Config directory: {:?}", PROJECT_DIRS.config_dir());

    let config = match config::Config::builder()
        .add_source(config::File::with_name(config_path.as_str()))
        .build()
    {
        Ok(config_loader) => {
            debug!("Configuration file found, deserializing");
            config_loader.try_deserialize::<Config>().map_err(|e| {
                error!("Failed to deserialize config: {}", e);
                e
            })?
        }
        Err(e) => {
            info!("No configuration file found ({}), using default config", e);
            debug!(
                "Default config will be used with {} fps refresh rate",
                Config::default().refresh_fps
            );
            Config::default()
        }
    };

    info!("Configuration loaded successfully");
    debug!(
        "Config settings - FPS: {}, Theme: {:?}, Article scope: {:?}",
        config.refresh_fps, config.theme, config.article_scope
    );

    Ok(config)
}
