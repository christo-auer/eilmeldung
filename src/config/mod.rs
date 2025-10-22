pub mod paths;
pub mod theme;
use std::collections::HashMap;

use crate::commands::{Command, CommandSequence};
use crate::config::paths::{CONFIG_FILE, PROJECT_DIRS};
use crate::config::theme::Theme;
use crate::input::KeySequence;
use crate::ui::articles_list::ArticleScope;
use log::{debug, error, info};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum ArticleContentType {
    PlainText,
    Markdown,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub input_config: InputConfig,
    pub theme: Theme,

    pub refresh_fps: u64,

    pub all_label: String,
    pub feed_label: String,
    pub category_label: String,
    pub tags_label: String,
    pub tag_label: String,
    pub tag_icon: char,
    pub article_table: String,
    pub date_format: String,
    pub read_icon: char,
    pub unread_icon: char,
    pub marked_icon: char,
    pub unmarked_icon: char,
    pub article_scope: ArticleScope,

    pub article_auto_scrape: bool,
    pub article_thumbnail_show: bool,
    pub article_thumbnail_width: u16,
    pub article_thumbnail_resize: bool,
    pub article_thumbnail_fetch_debounce_millis: u64,
    pub article_content_max_chars_per_line: u16,
    pub article_content_preferred_type: ArticleContentType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_label: "󱀂 All {unread_count}".into(),
            feed_label: " {label} {unread_count}".into(),
            category_label: "󰉋 {label} {unread_count}".into(),
            tags_label: "󰓻 Tags {unread_count}".into(),
            tag_label: "󰓹 {label} {unread_count}".into(),
            article_table: "{read},{marked},{tag_icons},{date},{title}".into(),
            date_format: "%m/%d %H:%M".into(),
            theme: Default::default(),
            input_config: Default::default(),
            refresh_fps: 10,
            read_icon: '',
            unread_icon: '',
            marked_icon: '',
            unmarked_icon: ' ',
            tag_icon: '󰓹',
            article_scope: ArticleScope::Unread,

            article_auto_scrape: true,
            article_thumbnail_show: true,
            article_thumbnail_width: 20,
            article_thumbnail_resize: false,
            article_thumbnail_fetch_debounce_millis: 500,
            article_content_max_chars_per_line: 66,
            article_content_preferred_type: ArticleContentType::Markdown,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InputConfig {
    pub scroll_amount: usize,
    pub input_timeout_millis: u64,
    pub input_commands: HashMap<KeySequence, CommandSequence>,
}

fn generate_default_input_commands() -> HashMap<KeySequence, CommandSequence> {
    use Command::*;
    vec![
        ("j".into(), NavigateDown.into()),
        ("C-f".into(), NavigatePageDown.into()),
        ("C-b".into(), NavigatePageUp.into()),
        ("g g".into(), NavigateFirst.into()),
        ("G".into(), NavigateLast.into()),
        ("k".into(), NavigateUp.into()),
        ("h".into(), NavigateLeft.into()),
        ("l".into(), NavigateRight.into()),
        ("q".into(), ApplicationQuit.into()),
        ("r".into(), FeedsSync.into()),
        ("s".into(), ArticleCurrentScrape.into()),
        ("g f".into(), PanelFocusFeeds.into()),
        ("g a".into(), PanelFocusArticleSelection.into()),
        ("g c".into(), PanelFocusArticleContent.into()),
        ("space".into(), PanelFocusNext.into()),
        ("backspace".into(), PanelFocusPrevious.into()),
        ("tab".into(), PanelFocusNextCyclic.into()),
        ("backtab".into(), PanelFocusPreviousCyclic.into()),
        (
            "o".into(),
            vec![
                ArticleCurrentOpenInBrowser,
                ArticleCurrentSetRead,
                ArticleListSelectNextUnread,
            ]
            .into(),
        ),
        (
            "n".into(),
            vec![ArticleCurrentSetRead, ArticleListSelectNextUnread].into(),
        ),
        ("u".into(), ArticleCurrentSetUnread.into()),
        ("U".into(), ArticleCurrentToggleRead.into()),
        ("a".into(), ArticleListSetAllRead.into()),
        ("A".into(), ArticleListSetAllUnread.into()),
        ("1".into(), ArticleListSetScope(ArticleScope::All).into()),
        ("2".into(), ArticleListSetScope(ArticleScope::Unread).into()),
        ("3".into(), ArticleListSetScope(ArticleScope::Marked).into()),
        ("z".into(), ToggleDistractionFreeMode.into()),
    ]
    .into_iter()
    .collect()
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_amount: 10,
            input_timeout_millis: 5000,
            input_commands: generate_default_input_commands(),
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
