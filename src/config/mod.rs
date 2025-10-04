pub mod paths;
pub mod theme;
use crate::config::paths::{CONFIG_FILE, PROJECT_DIRS};
use crate::config::theme::Theme;
use log::info;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub input_config: InputConfig,
    pub theme: Theme,
    pub all_label: String,
    pub feed_label: String,
    pub category_label: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_label: "󱀂 All {unread_count}".into(),
            feed_label: " {label} {unread_count}".into(),
            category_label: "󰉋 {label} {unread_count}".into(),
            theme: Default::default(),
            input_config: Default::default(),
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

    info!("loading config from {config_path}");

    let config = match config::Config::builder()
        .add_source(config::File::with_name(config_path.as_str()))
        .build()
    {
        Ok(config_loader) => config_loader.try_deserialize::<Config>()?,
        _ => {
            info!("no configuration file found. assuming default config");
            // TODO serialize default config?
            Config::default()
        }
    };

    Ok(config)
}
