pub mod input_config;
pub mod paths;
pub mod theme;

use std::str::FromStr;

use crate::prelude::*;

pub mod prelude {
    pub use super::input_config::InputConfig;
    pub use super::paths::{CONFIG_FILE, PROJECT_DIRS};
    pub use super::theme::Theme;
    pub use super::{ArticleContentType, ArticleScope, Config, LabeledQuery, load_config};
}

use log::{debug, error, info};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum ArticleContentType {
    PlainText,
    Markdown,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, serde::Deserialize)]
pub struct LabeledQuery {
    pub query: String,
    pub label: String,
}

impl From<(String, String)> for LabeledQuery {
    fn from((label, query): (String, String)) -> Self {
        Self { label, query }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ArticleScope {
    All,
    Unread,
    Marked,
}

impl FromStr for ArticleScope {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "all" => Self::All,
            "unread" => Self::Unread,
            "marked" => Self::Marked,
            _ => {
                return Err(color_eyre::eyre::eyre!("expected all, unread or marked"));
            }
        })
    }
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

    pub queries: Vec<LabeledQuery>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_label: "󱀂 All {unread_count}".into(),
            feed_label: " {label} {unread_count}".into(),
            category_label: "󰉋 {label} {unread_count}".into(),
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

            queries: vec![
                // ("Heise".into(), "unread feed:heise".into()).into(),
                ("Apple News".into(), "title:/(?i).*apple:.*/".into()).into(),
                ("Tag test ".into(), "tag:#test feedurl:\"pitchfork\"".into()).into(),
                ("Tag test 2".into(), "tag:#test,#abc ~tag:#abcd".into()).into(),
                (
                    "alle ungelesenen von heute auf heise".into(),
                    "newer:\"1 day ago\" ~newer:\"5 hours ago\" unread feed:/(?i)heise/".into(),
                )
                    .into(),
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
