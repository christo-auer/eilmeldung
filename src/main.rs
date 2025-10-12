use news_flash::{
    NewsFlash,
    models::{DirectLogin, LoginData, PluginID},
};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    commands::Command,
    config::{load_config, paths},
    newsflash_utils::NewsFlashAsyncManager,
};

mod app;
mod commands;
mod config;
mod input;
mod logging;
mod newsflash_utils;
mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    crate::logging::init_logging()?;

    let config = load_config()?;

    let config_dir = paths::PROJECT_DIRS.config_dir();
    let data_dir = paths::PROJECT_DIRS.data_dir();
    let local_plugin_id = PluginID::new("freshrss");
    let news_flash = NewsFlash::new(config_dir, data_dir, &local_plugin_id, None)?;
    let (command_sender, command_receiver) = unbounded_channel::<Command>();

    let login_data = LoginData::Direct(DirectLogin::Password(news_flash::models::PasswordLogin {
        id: local_plugin_id,
        url: Some("http://10.0.64.2:8081/api/greader.php".into()),
        user: "chris".into(),
        password: "abcdefgh".into(),
        basic_auth: None,
    }));

    let client = reqwest::Client::new();

    news_flash.login(login_data, &client).await?;

    let news_flash_async_manager =
        NewsFlashAsyncManager::new(news_flash, client, command_sender.clone());
    let app = crate::app::App::new(config, news_flash_async_manager, command_sender);

    let terminal = ratatui::init();

    let result = app.run(command_receiver, terminal).await;

    ratatui::restore();

    result
}
