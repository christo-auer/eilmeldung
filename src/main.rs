mod cli;
mod config;
mod connectivity;
mod input;
mod logging;
mod login;
mod messages;
mod newsflash_utils;
mod query;
mod ui;
mod utils;

use std::{path::Path, sync::Arc, time::Duration};

use clap::Parser;
use log::{debug, error, info};
use news_flash::{NewsFlash, models::LoginData};
use tokio::{sync::mpsc::unbounded_channel, task::spawn_blocking};

mod prelude;
use crate::{connectivity::ConnectivityMonitor, prelude::*};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    info!("Starting eilmeldung application");

    info!("parsing CLI args");
    let cli_args = CliArgs::parse();
    let config_dir = cli_args
        .config_dir()
        .as_ref()
        .map(Path::new)
        .unwrap_or(PROJECT_DIRS.config_dir());
    let state_dir = cli_args
        .state_dir()
        .as_ref()
        .map(Path::new)
        .unwrap_or(PROJECT_DIRS.state_dir().unwrap_or(PROJECT_DIRS.data_dir()));

    color_eyre::install()?;
    crate::logging::init_logging(&cli_args)?;
    debug!("Error handling and logging initialized");

    info!("Loading configuration");
    let config = Arc::new(load_config(config_dir)?);

    info!("Initializing NewsFlash");
    let news_flash_attempt = NewsFlash::try_load(state_dir, config_dir);

    let client = build_client(Duration::from_secs(config.network_timeout_seconds))?;

    let news_flash = match news_flash_attempt {
        Ok(news_flash) => news_flash,
        Err(_) => {
            // this is the initial setup => setup login data
            info!("no profile found => ask user or try config");
            let mut logged_in = false;
            // skip if login configuration is given
            let mut skip_asking_for_login = config.login_setup.is_some();

            let mut login_data: Option<LoginData> = config
                .login_setup
                .as_ref()
                .inspect(|_| info!("login configuration found"))
                .map(|login_configuration| login_configuration.to_login_data())
                .transpose()?;
            let login_setup = LoginSetup::new();
            let mut news_flash: Option<NewsFlash> = None;
            while !logged_in {
                login_data = if login_data.is_none() || !skip_asking_for_login {
                    skip_asking_for_login = false;
                    Some(login_setup.inquire_login_data(&login_data).await?)
                } else {
                    login_data
                };
                news_flash = Some(NewsFlash::new(
                    state_dir,
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

    // just print login data and exit
    if *cli_args.print_login_data() {
        let login_data = news_flash.get_login_data().await.unwrap();
        print!(
            "{}",
            LoginConfiguration::from(login_data).as_toml(*cli_args.show_secrets())?
        );
        return Ok(());
    }

    // setup of things we need in the app
    let (message_sender, message_receiver) = unbounded_channel::<Message>();
    let input_reader_message_sender = message_sender.clone();
    let news_flash_utils = Arc::new(NewsFlashUtils::new(
        news_flash,
        client,
        config.clone(),
        message_sender.clone(),
    ));
    let connectivity_monitor =
        ConnectivityMonitor::new(news_flash_utils.clone(), message_sender.clone());

    // create the main app
    let app = App::new(config, news_flash_utils.clone(), message_sender);

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
