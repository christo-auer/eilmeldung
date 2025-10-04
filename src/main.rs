use news_flash::{NewsFlash, models::PluginID};

use crate::config::{load_config, paths};

mod app;
mod commands;
mod config;
mod input;
mod logging;
mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    crate::logging::init_logging()?;

    let config = load_config()?;

    let config_dir = paths::PROJECT_DIRS.config_dir();
    let data_dir = paths::PROJECT_DIRS.data_dir();
    let local_plugin_id = PluginID::new("local_rss");
    let news_flash = NewsFlash::new(&config_dir, &data_dir, &local_plugin_id, None)?;
    let client = reqwest::Client::new();

    let terminal = ratatui::init();
    let result = crate::app::App::new(config, news_flash, client)
        .run(terminal)
        .await;

    ratatui::restore();

    result
}
