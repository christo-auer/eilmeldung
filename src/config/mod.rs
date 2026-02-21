mod cli_config;
mod dimension;
mod feed_list_content_identfier;
mod input_config;
mod login_configuration;
mod paths;
mod share_target;
mod theme;

use std::path::{Path, PathBuf};

use crate::prelude::*;

pub mod prelude {
    pub use super::cli_config::CliConfig;
    pub use super::dimension::Dimension;
    pub use super::feed_list_content_identfier::{
        FeedListContentIdentifier, FeedListItemType, LabeledQuery,
    };
    pub use super::input_config::InputConfig;
    pub use super::login_configuration::LoginConfiguration;
    pub use super::paths::{CONFIG_FILE, PROJECT_DIRS};
    pub use super::share_target::ShareTarget;
    pub use super::theme::Theme;
    pub use super::{ArticleContentType, ArticleScope, Config, ConfigError, load_config};
}

use config::FileFormat;
use log::{info, warn};
use ratatui::symbols::scrollbar;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("configuration could not be validated")]
    ValidationError(String),
    #[error("feed list content identifier could not be parsed")]
    FeedListContentIdentifierParseError(String),
    #[error("share target could not be parsed")]
    ShareTargetParseError(String),
    #[error("dimension could not be parsed")]
    DimensionParseError(String),
    #[error("invalid URL template for share target")]
    ShareTargetInvalidUrlError(#[from] url::ParseError),
    #[error("invalid target")]
    ShareTargetInvalid,
    #[error("invalid share command")]
    ShareTargetInvalidCommand(#[from] shell_words::ParseError),
    #[error("invalid secret or secret command")]
    SecretParseError,
    #[error("invalid secret or secret command")]
    SecretCommandExecutionError(String),
    #[error("invalid login configuration")]
    LoginConfigurationInvalid(String),
}

#[derive(Debug, Clone, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
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

impl ArticleScope {
    pub fn to_icon(self, config: &Config) -> char {
        use ArticleScope as A;
        match self {
            A::All => config.all_icon,
            A::Unread => config.unread_icon,
            A::Marked => config.marked_icon,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub input_config: InputConfig,
    pub theme: Theme,
    pub refresh_fps: u64,
    pub network_timeout_seconds: u64,
    pub keep_articles_days: u16,

    pub startup_commands: Vec<Command>,

    pub sync_every_minutes: Option<u64>,

    pub after_sync_commands: Vec<Command>,

    pub offline_icon: char,
    pub all_label: String,
    pub last_synced_label: String,
    pub feed_label: String,
    pub category_label: String,
    pub categories_label: String,
    pub tags_label: String,
    pub tag_label: String,
    pub query_label: String,
    pub all_icon: char,
    pub tag_icon: char,
    pub info_icon: char,
    pub warning_icon: char,
    pub error_icon: char,
    pub article_table: String,
    pub date_format: String,
    pub read_icon: char,
    pub unread_icon: char,
    pub marked_icon: char,
    pub unmarked_icon: char,
    pub enclosure_video_icon: char,
    pub enclosure_audio_icon: char,
    pub enclosure_image_icon: char,
    pub command_line_prompt_icon: char,
    pub article_scope: ArticleScope,
    pub feed_list_scope: ArticleScope,
    pub scrollbar_begin_symbol: char,
    pub scrollbar_end_symbol: char,
    pub scrollbar_track_symbol: char,
    pub scrollbar_thumb_symbol: char,

    pub articles_after_selection: usize,
    pub auto_scrape: bool,
    pub thumbnail_show: bool,
    pub thumbnail_width: Dimension,
    pub thumbnail_height: Dimension,
    pub thumbnail_resize: bool,
    pub thumbnail_fetch_debounce_millis: u64,
    pub text_max_width: u16,
    pub content_preferred_type: ArticleContentType,
    pub hide_default_sort_order: bool,
    pub default_sort_order: SortOrder,
    pub zen_mode_show_header: bool,

    pub feed_list_focused_width: Dimension,
    pub article_list_focused_width: Dimension,
    pub article_list_focused_height: Dimension,
    pub article_content_focused_height: Dimension,

    pub enclosure_command: String,
    pub video_enclosure_command: Option<String>,
    pub audio_enclosure_command: Option<String>,
    pub image_enclosure_command: Option<String>,

    pub feed_list: Vec<FeedListContentIdentifier>,

    pub share_targets: Vec<ShareTarget>,

    pub login_setup: Option<LoginConfiguration>,

    pub cli: CliConfig,
}

impl Config {
    fn validate(&mut self) -> color_eyre::Result<()> {
        self.validate_input_config()?;

        if let Some(sync_interval) = self.sync_every_minutes
            && sync_interval == 0
        {
            return Err(color_eyre::eyre::eyre!(
                "sync_every_minutes must at least be 1"
            ));
        }

        Ok(())
    }

    fn validate_input_config(&mut self) -> color_eyre::Result<()> {
        Self::default()
            .input_config
            .mappings
            .into_iter()
            .for_each(|(key_seq, cmd_seq)| {
                self.input_config.mappings.entry(key_seq).or_insert(cmd_seq);
            });

        self.input_config
            .mappings
            .iter()
            .filter_map(|(key_seq, command_seq)| command_seq.commands.is_empty().then_some(key_seq))
            .cloned()
            .collect::<Vec<KeySequence>>()
            .into_iter()
            .for_each(|key| {
                self.input_config.mappings.shift_remove(&key);
            });

        Ok(())
    }

    pub fn scrollbar_set(&self) -> scrollbar::Set<'_> {
        scrollbar::Set {
            track: Box::new(self.scrollbar_track_symbol.to_string()).leak(),
            thumb: Box::new(self.scrollbar_thumb_symbol.to_string()).leak(),
            begin: Box::new(self.scrollbar_begin_symbol.to_string()).leak(),
            end: Box::new(self.scrollbar_end_symbol.to_string()).leak(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_fps: 10,
            network_timeout_seconds: 60,
            keep_articles_days: 30,

            startup_commands: Default::default(),
            sync_every_minutes: None,

            after_sync_commands: Default::default(),

            all_label: "󱀂 All {unread_count}".into(),
            last_synced_label: " Last Synced".into(),
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
            offline_icon: '',
            read_icon: '',
            all_icon: '',
            unread_icon: '',
            marked_icon: '',
            unmarked_icon: ' ',
            tag_icon: '󰓹',
            command_line_prompt_icon: '',
            info_icon: '',
            warning_icon: '',
            error_icon: '',
            enclosure_video_icon: '',
            enclosure_audio_icon: '',
            enclosure_image_icon: '',
            article_scope: ArticleScope::Unread,
            feed_list_scope: ArticleScope::All,
            scrollbar_begin_symbol: '│',
            scrollbar_end_symbol: '│',
            scrollbar_track_symbol: ' ',
            scrollbar_thumb_symbol: '│',

            articles_after_selection: 3,
            auto_scrape: true,
            thumbnail_show: true,
            thumbnail_width: Dimension::Length(14),
            thumbnail_height: Dimension::Length(5),
            thumbnail_resize: true,
            thumbnail_fetch_debounce_millis: 500,
            text_max_width: 66,
            content_preferred_type: ArticleContentType::Markdown,
            zen_mode_show_header: false,

            feed_list_focused_width: Dimension::Percentage(25),
            article_list_focused_width: Dimension::Percentage(75),
            article_list_focused_height: Dimension::Percentage(20),
            article_content_focused_height: Dimension::Percentage(80),

            default_sort_order: SortOrder::new(vec![SortKey::Date(SortDirection::Ascending)]),
            hide_default_sort_order: true,

            #[cfg(target_os = "macos")]
            enclosure_command: "open {url}".into(),

            #[cfg(target_os = "linux")]
            enclosure_command: "xdg-open {url}".into(),

            video_enclosure_command: None,
            audio_enclosure_command: None,
            image_enclosure_command: None,

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
            login_setup: None,
            cli: CliConfig::default(),
        }
    }
}

pub fn load_config(config_dir: &Path) -> color_eyre::Result<Config> {
    let mut config_path = PathBuf::from(config_dir);
    config_path.push(CONFIG_FILE);

    let Some(config_path) = config_path.to_str() else {
        return Err(color_eyre::eyre::eyre!("invalid configuration path"));
    };

    info!("Trying to load config from {}", config_path);

    if !Path::new(config_path).exists() {
        info!("No config file found, using default config");
        return Ok(Config::default());
    }

    let mut config = match config::Config::builder()
        .add_source(config::File::new(config_path, FileFormat::Toml))
        .build()
    {
        Ok(config) => config.try_deserialize::<Config>()?,
        Err(err) => {
            warn!("unable to read config file: {err}");
            return Err(color_eyre::eyre::eyre!(err));
        }
    };

    config.validate()?;

    Ok(config)
}
