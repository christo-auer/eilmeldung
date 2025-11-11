use std::sync::Arc;

use log::{debug, error, info};
use news_flash::{NewsFlash, models::LoginData};
use tokio::{sync::mpsc::unbounded_channel, task::spawn_blocking};

mod prelude;
use crate::{connectivity::ConnectivityMonitor, prelude::*};

mod app;
mod config;
mod connectivity;
mod input;
mod logging;
mod login;
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

    let config_dir = PROJECT_DIRS.config_dir();
    let state_dir = PROJECT_DIRS.state_dir();

    info!("Initializing NewsFlash");
    let news_flash_attempt = NewsFlash::try_load(state_dir.unwrap(), config_dir);

    let client = reqwest::Client::new();

    let news_flash = match news_flash_attempt {
        Ok(news_flash) => news_flash,
        Err(_) => {
            // this is the initial setup => setup login data
            info!("no profile found => ask user");
            let mut logged_in = false;

            let mut login_data: Option<LoginData> = None;
            let login_setup = LoginSetup::new();
            let mut news_flash: Option<NewsFlash> = None;
            while !logged_in {
                login_data = Some(login_setup.inquire_login_data(&login_data).await?);
                news_flash = Some(NewsFlash::new(
                    state_dir.unwrap(),
                    config_dir,
                    &login_data.as_ref().unwrap().id(),
                    None,
                )?);
                logged_in = login_setup
                    .login_and_initial_sync(
                        news_flash.as_ref().unwrap(),
                        login_data.as_ref().unwrap(),
                        &client,
                    )
                    .await?;
            }
            news_flash.unwrap()
        }
    };

    // setup of things we need in the app
    let (message_sender, message_receiver) = unbounded_channel::<Message>();
    let input_reader_message_sender = message_sender.clone();
    let news_flash_utils = Arc::new(NewsFlashUtils::new(
        news_flash,
        client,
        message_sender.clone(),
    ));
    let connectivity_monitor =
        ConnectivityMonitor::new(news_flash_utils.clone(), message_sender.clone());

    // create the main app
    let app = crate::app::App::new(config, news_flash_utils.clone(), message_sender);

    info!("Initializing terminal");
    let terminal = ratatui::init();

    // startup task which reads the crossterm events
    let _input_reader_handle = spawn_blocking(move || {
        if let Err(err) = input_reader(input_reader_message_sender) {
            error!("input reader got an error: {err}");
        }
    });

    let _connecitivty_monitor_handle = connectivity_monitor.spawn()?;

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
