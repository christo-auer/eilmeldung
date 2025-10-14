pub mod paths;
pub mod theme;
use crate::config::paths::{CONFIG_FILE, PROJECT_DIRS};
use crate::config::theme::Theme;
use crate::ui::articles_list::ArticleScope;
use log::{error, info, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum ArticleContentType {
    PlainText,
    Markdown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub input_config: InputConfig,
    pub theme: Theme,

    pub refresh_fps: u64,

    pub all_label: String,
    pub feed_label: String,
    pub category_label: String,
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
    pub article_content_max_chars_per_line: u16,
    pub article_content_preferred_type: ArticleContentType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_label: "󱀂 All {unread_count}".into(),
            feed_label: " {label} {unread_count}".into(),
            category_label: "󰉋 {label} {unread_count}".into(),
            article_table: "{read},{marked},{date},{title}".into(),
            date_format: "%m/%d %H:%M".into(),
            theme: Default::default(),
            input_config: Default::default(),
            refresh_fps: 10,
            read_icon: '',
            unread_icon: '',
            marked_icon: '',
            unmarked_icon: ' ',
            article_scope: ArticleScope::Unread,

            article_auto_scrape: true,
            article_thumbnail_show: true,
            article_thumbnail_width: 20,
            article_thumbnail_resize: false,
            article_content_max_chars_per_line: 66,
            article_content_preferred_type: ArticleContentType::Markdown,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InputConfig {
    pub scroll_amount: usize,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self { scroll_amount: 5 }
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
            config_loader.try_deserialize::<Config>()
                .map_err(|e| {
                    error!("Failed to deserialize config: {}", e);
                    e
                })?
        },
        Err(e) => {
            info!("No configuration file found ({}), using default config", e);
            debug!("Default config will be used with {} fps refresh rate", Config::default().refresh_fps);
            Config::default()
        }
    };

    info!("Configuration loaded successfully");
    debug!("Config settings - FPS: {}, Theme: {:?}, Article scope: {:?}", 
           config.refresh_fps, config.theme, config.article_scope);

    Ok(config)
}
