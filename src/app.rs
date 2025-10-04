use news_flash::NewsFlash;
use ratatui::DefaultTerminal;
use std::sync::Arc;
use tokio::sync::{
    RwLock,
    mpsc::{UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
};

use crate::{
    commands::Command, config::Config, input::translate_to_command, ui::feeds_list::FeedList,
};

pub struct App {
    pub config: Arc<Config>,
    pub news_flash: Arc<RwLock<NewsFlash>>,
    pub client: Arc<RwLock<reqwest::Client>>,

    pub feed_list: FeedList,

    pub is_running: bool,
}

impl App {
    pub fn new(config: Config, news_flash: NewsFlash, client: reqwest::Client) -> Self {
        let config_arc = Arc::new(config);
        let news_flash_arc = Arc::new(RwLock::new(news_flash));
        Self {
            config: Arc::clone(&config_arc),
            news_flash: Arc::clone(&news_flash_arc),
            client: Arc::new(RwLock::new(client)),
            is_running: true,
            feed_list: FeedList::new(Arc::clone(&config_arc), Arc::clone(&news_flash_arc)),
        }
    }

    pub async fn run(mut self, terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let (input_tx, input_rx) = unbounded_channel::<Command>();

        // TODO build tree via command
        self.feed_list.build_tree().await?;

        let input_config = self.config.clone();
        tokio::spawn(
            async move { App::process_input(input_config.clone(), input_tx.clone()).await },
        );

        self.process_commands(input_rx, terminal).await?;

        Ok(())
    }

    async fn process_input(
        config: Arc<Config>,
        tx: UnboundedSender<Command>,
    ) -> color_eyre::Result<()> {
        loop {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => {
                    if let Some(command) = translate_to_command(&config.input_config, key_event) {
                        tx.send(command)?
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }

    async fn process_commands(
        mut self,
        mut rx: UnboundedReceiver<Command>,
        mut terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        while self.is_running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            use Command::*;
            match rx.recv().await {
                Some(ApplicationQuit) => self.is_running = false,
                Some(command) => {
                    // TODO queue commands
                    self.feed_list.process_command(command);
                }
                None => {}
            }
        }

        Ok(())
    }
}
