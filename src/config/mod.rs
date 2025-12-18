pub mod feed_list_content_identfier;
pub mod input_config;
pub mod paths;
pub mod share_target;
pub mod theme;

use std::path::{Path, PathBuf};

use crate::prelude::*;

pub mod prelude {
    pub use super::feed_list_content_identfier::{
        FeedListContentIdentifier, FeedListItemType, LabeledQuery,
    };
    pub use super::input_config::InputConfig;
    pub use super::paths::{CONFIG_FILE, PROJECT_DIRS};
    pub use super::share_target::ShareTarget;
    pub use super::theme::Theme;
    pub use super::{ArticleContentType, ArticleScope, Config, ConfigError, load_config};
}

use config::FileFormat;
use log::{info, warn};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("configuration could not be validated")]
    ValidationError(String),
    #[error("feed list content identifier could not be parsed")]
    FeedListContentIdentifierParseError(String),
    #[error("share target could not be parsed")]
    ShareTargetParseError(String),
    #[error("invalid URL template for share target")]
    ShareTargetInvalidUrlError(#[from] url::ParseError),
    #[error("invalid target")]
    ShareTargetInvalid,
}

#[derive(Debug, Clone, serde::Deserialize, Eq, PartialEq)]
pub enum ArticleContentType {
    PlainText,
    Markdown,
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
    strum::EnumMessage,
    strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
pub enum ArticleScope {
    #[default]
    #[strum(serialize = "all", message = "all", detailed_message = "all articles")]
    All,
    #[strum(
        serialize = "unread",
        message = "unread",
        detailed_message = "only unread articles"
    )]
    Unread,
    #[strum(
        serialize = "marked",
        message = "marked",
        detailed_message = "only marked articles"
    )]
    Marked,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
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

    pub articles_after_selection: usize,
    pub auto_scrape: bool,
    pub thumbnail_show: bool,
    pub thumbnail_width: u16,
    pub thumbnail_resize: bool,
    pub thumbnail_fetch_debounce_millis: u64,
    pub text_max_width: u16,
    pub content_preferred_type: ArticleContentType,

    pub feed_list_width_percent: u16,
    pub article_list_width_percent: u16,
    pub article_list_height_lines: u16,

    pub feed_list: Vec<FeedListContentIdentifier>,

    pub share_targets: Vec<ShareTarget>,
}

impl Config {
    fn validate(&mut self) -> color_eyre::Result<()> {
        self.input_config.validate()?;

        Ok(())
    }
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

            articles_after_selection: 3,
            auto_scrape: true,
            thumbnail_show: true,
            thumbnail_width: 20,
            thumbnail_resize: true,
            thumbnail_fetch_debounce_millis: 500,
            text_max_width: 66,
            content_preferred_type: ArticleContentType::Markdown,

            feed_list_width_percent: 33,
            article_list_width_percent: 67,
            article_list_height_lines: 6,

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

            share_targets: vec![
                ShareTarget::Clipboard,
                ShareTarget::Reddit,
                ShareTarget::Mastodon,
                ShareTarget::Instapaper,
                ShareTarget::Telegram,
            ],
        }
    }
}

pub fn load_config(config_dir: &Path) -> color_eyre::Result<Config> {
    let mut config_path = PathBuf::from(config_dir);
    config_path.push(CONFIG_FILE);

    let Some(config_path) = config_path.to_str() else {
        return Err(color_eyre::eyre::eyre!("invalid configuration path"));
    };

    info!("Loading config from {}", config_path);

    let mut config = match config::Config::builder()
        .add_source(config::File::new(config_path, FileFormat::Toml))
        .build()
    {
        Ok(config) => config.try_deserialize::<Config>()?,
        Err(err) => {
            warn!("unable to read config file: {err}");
            Config::default()
        }
    };

    config.validate()?;

    Ok(config)
}
