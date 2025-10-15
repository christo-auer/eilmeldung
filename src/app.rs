use log::{debug, error, info, trace};
use ratatui::DefaultTerminal;
use std::{sync::Arc, time::Duration};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    commands::{Command, Event, Message, MessageReceiver},
    config::Config,
    input::translate_to_commands,
    newsflash_utils::NewsFlashAsyncManager,
    ui::{
        article_content::ArticleContent,
        articles_list::ArticlesList,
        feeds_list::FeedList,
        tooltip::{Tooltip, TooltipFlavor},
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AppState {
    FeedSelection,
    ArticleSelection,
    ArticleContent,
    ArticleContentDistractionFree,
}

impl AppState {
    fn previous_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            FeedSelection => ArticleContent,
            ArticleContentDistractionFree => ArticleContentDistractionFree,
        }
    }

    fn next_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => FeedSelection,
            ArticleContentDistractionFree => ArticleContentDistractionFree,
        }
    }

    fn next(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => ArticleContent,
            ArticleContentDistractionFree => ArticleContentDistractionFree,
        }
    }

    fn previous(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => FeedSelection,
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            ArticleContentDistractionFree => ArticleContentDistractionFree,
        }
    }
}

pub struct App {
    pub state: AppState,

    pub config: Arc<Config>,
    pub news_flash_async_manager: Arc<NewsFlashAsyncManager>,
    pub message_sender: UnboundedSender<Message>,

    pub tooltip: Tooltip,

    pub feed_list: FeedList,
    pub articles_list: ArticlesList,
    pub article_content: ArticleContent,
    pub async_operation_throbber: ThrobberState,

    pub is_running: bool,
}

impl App {
    pub fn new(
        config: Config,
        news_flash_async_manager: NewsFlashAsyncManager,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating new App instance");
        let config_arc = Arc::new(config);
        let news_flash_async_manager_arc = Arc::new(news_flash_async_manager);

        debug!("Initializing UI components");
        let app = Self {
            state: AppState::FeedSelection,
            config: Arc::clone(&config_arc),
            news_flash_async_manager: news_flash_async_manager_arc.clone(),
            is_running: true,
            message_sender: message_sender.clone(),
            feed_list: FeedList::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                message_sender.clone(),
            ),
            articles_list: ArticlesList::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                message_sender.clone(),
            ),
            article_content: ArticleContent::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                message_sender.clone(),
            ),
            tooltip: Tooltip::new(
                "Welcome to eilmeldung".into(),
                crate::ui::tooltip::TooltipFlavor::Info,
            ),
            async_operation_throbber: ThrobberState::default(),
        };

        info!("App instance created with initial state: FeedSelection");
        app
    }

    pub async fn run(
        mut self,
        message_receiver: UnboundedReceiver<Message>,
        terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        info!("Starting application run loop");

        debug!("Building feed tree");
        self.feed_list.build_tree().await.map_err(|e| {
            error!("Failed to build feed tree: {}", e);
            e
        })?;
        info!("Feed tree built successfully");

        let input_config = self.config.clone();
        let input_tx = self.message_sender.clone();
        debug!("Spawning input processing task");
        tokio::spawn(async move {
            if let Err(e) = App::process_input(input_config.clone(), input_tx).await {
                error!("Input processing task failed: {}", e);
            }
        });

        debug!("Sending ApplicationStarted command");
        self.message_sender
            .send(Message::Event(Event::ApplicationStarted))?;

        info!("Starting command processing loop");
        self.process_commands(message_receiver, terminal).await?;

        info!("Application run loop completed");
        Ok(())
    }

    fn tick(&mut self) {
        if self.news_flash_async_manager.is_async_operation_running() {
            trace!("Async operation running, updating throbber");
            self.async_operation_throbber.calc_next();
        }
    }

    async fn process_input(
        config: Arc<Config>,
        tx: UnboundedSender<Message>,
    ) -> color_eyre::Result<()> {
        debug!("Input processing loop started");
        loop {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => {
                    trace!("Key event received: {:?}", key_event);
                    let commands = translate_to_commands(&config.input_config, key_event);
                    debug!("Translated to {} commands", commands.len());
                    for command in commands {
                        // Don't log commands with large binary data
                        tx.send(command).map_err(|e| {
                            error!("Failed to send command: {}", e);
                            e
                        })?;
                    }
                }
                _ => {
                    trace!("Non-key event ignored");
                }
            }
        }
    }

    async fn process_commands(
        mut self,
        mut rx: UnboundedReceiver<Message>,
        mut terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        let mut render_interval =
            tokio::time::interval(Duration::from_millis(1000 / self.config.refresh_fps));
        debug!(
            "Command processing loop started with {}fps refresh rate",
            self.config.refresh_fps
        );

        while self.is_running {
            tokio::select! {
                _ = render_interval.tick() => {
                    self.message_sender.send(Message::Event(Event::Tick))?;

                }

                message = rx.recv() =>  {
                    if let Some(message) = message {
                        match &message {
                            Message::Event(Event::Tick) => { /* don't spam the log */ },
                            Message::Event(Event::AsyncFetchThumbnailFinished(_)) => {
                                trace!("Processing message: AsyncFetchThumbnailFinished");
                            }
                            _ => {
                                trace!("Processing message: {:?}", message);
                            }
                        }

                        if let Err(e) = self.process_command(&message).await {
                            error!("Failed to process app message: {}", e);
                        }

                        if let Err(e) = self.feed_list.process_command(&message).await {
                            error!("Failed to process feed list message: {}", e);
                        }

                        if let Err(e) = self.articles_list.process_command(&message).await {
                            error!("Failed to process articles list message: {}", e);
                        }

                        if let Err(e) = self.article_content.process_command(&message).await {
                            error!("Failed to process article content message: {}", e);
                        }
                    } else {
                        debug!("Message channel closed, stopping message processing");
                        break;
                    }

                    if let Err(e) = terminal.draw(|frame| frame.render_widget(&mut self, frame.area())) {
                        error!("Failed to render terminal: {}", e);
                    }
                }
            }
        }

        info!("Message processing loop ended");
        Ok(())
    }
}

impl MessageReceiver for App {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;
        match message {
            Message::Command(ApplicationQuit) => {
                info!("Application quit requested");
                self.is_running = false;
            }
            Message::Command(FeedsSync) => {
                info!("Sync operation initiated");
                self.news_flash_async_manager.sync_feeds();
            }

            Message::Event(Tooltip(tooltip)) => {
                trace!("Tooltip updated");
                self.tooltip = tooltip.clone();
            }

            Message::Event(Tick) => {
                self.tick();
            }

            Message::Event(AsyncOperationFailed(error)) => {
                error!("Async operation failed: {}", error);
                self.tooltip =
                    crate::ui::tooltip::Tooltip::new(error.clone(), TooltipFlavor::Error);
            }

            Message::Command(PanelFocusNext) => {
                let old_state = self.state;
                self.state = self.state.next();
                debug!("Focus moved from {:?} to {:?}", old_state, self.state);
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
            }

            Message::Command(PanelFocusPrevious) => {
                let old_state = self.state;
                self.state = self.state.previous();
                debug!("Focus moved from {:?} to {:?}", old_state, self.state);
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
            }

            Message::Command(PanelFocusNextCyclic) => {
                let old_state = self.state;
                self.state = self.state.next_cyclic();
                debug!(
                    "Cyclic focus moved from {:?} to {:?}",
                    old_state, self.state
                );
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
            }

            Message::Command(PanelFocusPreivousCyclic) => {
                let old_state = self.state;
                self.state = self.state.previous_cyclic();
                debug!(
                    "Cyclic focus moved from {:?} to {:?}",
                    old_state, self.state
                );
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
            }

            Message::Command(ToggleDistractionFreeMode) => {
                let old_state = self.state;
                self.state = match old_state {
                    AppState::ArticleContentDistractionFree => AppState::ArticleContent,
                    _ => AppState::ArticleContentDistractionFree,
                };
                debug!(
                    "Toggling distraction free state from {:?} to {:?}",
                    old_state, self.state
                );
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
            }

            _ => {
                // Don't log commands with large binary data
                match message {
                    Message::Event(AsyncFetchThumbnailFinished(_)) => {
                        trace!("Unhandled message in App: AsyncFetchThumbnailFinished");
                    }
                    _ => {
                        trace!("Unhandled message in App: {:?}", message);
                    }
                }
            }
        }

        Ok(())
    }
}
