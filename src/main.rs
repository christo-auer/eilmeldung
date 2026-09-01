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
mod undo;
mod utils;

use std::{sync::Arc, time::Duration};

use clap::Parser;
use log::{debug, error, info};
use ratatui::crossterm::{event::DisableMouseCapture, execute};
use tokio::{sync::mpsc::unbounded_channel, task::spawn_blocking};

mod prelude;
use crate::{connectivity::ConnectivityMonitor, prelude::*};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli_args = CliArgs::parse();

    // color_eyre::install()?;
    crate::logging::init_logging(&cli_args)?;
    debug!("Error handling and logging initialized");

    // create channel
    let (message_sender, message_receiver) = unbounded_channel::<Message>();
    let (term_event_sender, term_event_receiver) =
        unbounded_channel::<ratatui::crossterm::event::Event>();

    info!("Loading configuration");
    let mut config_file_manager = ConfigFileManager::build(&cli_args, message_sender.clone());
    let mut config = config_file_manager.load_config().await?;

    info!("Initializing NewsFlash");
    let client = build_client(Duration::from_secs(config.network_timeout_seconds))?;

    let news_flash = login_news_flash(&client, &cli_args, &config).await?;

    // execute CLI actions -> if true, exit after execution (CLI only)
    if execute_cli_actions(&config, &cli_args, &news_flash, &client).await? {
        return Ok(());
    }

    // validate now
    config_file_manager.validate_config(&mut config).await?;

    // setup of things we need in the app
    let news_flash_utils = Arc::new(NewsFlashUtils::new(
        news_flash,
        client,
        message_sender.clone(),
    ));
    let connectivity_monitor =
        ConnectivityMonitor::new(news_flash_utils.clone(), message_sender.clone());

    // startup task which reads the crossterm events
    let input_reader_message_sender = message_sender.clone();
    let _input_reader_handle = spawn_blocking(move || {
        if let Err(err) = input_reader(input_reader_message_sender, term_event_sender) {
            error!("input reader got an error: {err}");
        }
    });

    // create the main app
    let app = App::new(
        config,
        config_file_manager,
        news_flash_utils.clone(),
        message_sender,
    );

    info!("Initializing terminal");
    let terminal = ratatui::init();

    // starting connectivity monitor
    let _connecitivty_monitor_handle = connectivity_monitor.spawn()?;

    info!("Starting application main loop");
    let result = app
        .run(term_event_receiver, message_receiver, terminal)
        .await;

    info!("Application loop ended, restoring terminal");
    execute!(std::io::stdout(), DisableMouseCapture)?;
    ratatui::restore();

    match &result {
        Ok(_) => info!("Application exited successfully"),
        Err(e) => error!("Application exited with error: {}", e),
    }

    result
}
