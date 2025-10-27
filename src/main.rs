use log::{debug, error, info};
use news_flash::{
    NewsFlash,
    models::{DirectLogin, LoginData, PluginID},
};
use tokio::sync::mpsc::unbounded_channel;

pub mod prelude {
    pub use super::app::{App, AppState};
    pub use super::config::prelude::*;
    pub use super::input::prelude::*;
    pub use super::logging::init_logging;
    pub use super::messages::prelude::*;
    pub use super::newsflash_utils::NewsFlashUtils;
    pub use super::query::AugmentedArticleFilter;
    pub use super::ui::prelude::*;
}

use crate::prelude::*;

mod app;
mod config;
mod input;
mod logging;
mod messages;
mod newsflash_utils;
mod query;
mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    info!("Starting eilmeldung application");

    color_eyre::install()?;
    crate::logging::init_logging()?;
    debug!("Error handling and logging initialized");

    info!("Loading configuration");
    let config = load_config()?;
    debug!("Configuration loaded successfully");

    let config_dir = PROJECT_DIRS.config_dir();
    let state_dir = PROJECT_DIRS.state_dir();

    let local_plugin_id = PluginID::new("freshrss");
    info!("Initializing NewsFlash with plugin: {}", local_plugin_id);

    let news_flash = NewsFlash::new(config_dir, state_dir.unwrap(), &local_plugin_id, None)
        .map_err(|e| {
            error!("Failed to initialize NewsFlash: {}", e);
            e
        })?;
    debug!("NewsFlash instance created successfully");

    let (message_sender, message_receiver) = unbounded_channel::<Message>();
    debug!("Message channel created");

    let login_data = LoginData::Direct(DirectLogin::Password(news_flash::models::PasswordLogin {
        id: local_plugin_id,
        url: Some("http://10.0.64.2:8081/api/greader.php".into()),
        user: "chris".into(),
        password: "abcdefgh".into(),
        basic_auth: None,
    }));

    let client = reqwest::Client::new();

    // info!("Attempting to login to NewsFlash");
    // news_flash.login(login_data, &client).await.map_err(|e| {
    //     error!("Failed to login to NewsFlash: {}", e);
    //     e
    // })?;

    // news_flash.set_offline(false, &client).await?;

    // news_flash.initial_sync(&client, Default::default()).await?;

    let news_flash_utils = NewsFlashUtils::new(news_flash, client, message_sender.clone());

    let app = crate::app::App::new(config, news_flash_utils, message_sender);

    info!("Initializing terminal");
    let terminal = ratatui::init();

    info!("Starting application main loop");
    let result = app.run(message_receiver, terminal).await;

    info!("Application loop ended, restoring terminal");
    ratatui::restore();

    match &result {
        Ok(_) => info!("Application exited successfully"),
        Err(e) => error!("Application exited with error: {}", e),
    }

    result
}
