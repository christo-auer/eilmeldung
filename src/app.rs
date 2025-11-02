use crate::prelude::*;

use log::{debug, error, info, trace};
use ratatui::DefaultTerminal;
use std::{fmt::Display, sync::Arc, time::Duration};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum AppState {
    FeedSelection,
    ArticleSelection,
    ArticleContent,
    ArticleContentDistractionFree,
}

impl Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppState::FeedSelection => write!(f, "feed selection"),
            AppState::ArticleSelection => write!(f, "article selection"),
            AppState::ArticleContent => write!(f, "article content"),
            AppState::ArticleContentDistractionFree => {
                write!(f, "article content distraction free")
            }
        }
    }
}

impl AppState {
    fn previous_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            FeedSelection => ArticleContent,
            _ => *self,
        }
    }

    fn next_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => FeedSelection,
            _ => *self,
        }
    }

    fn next(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => ArticleContent,
            _ => *self,
        }
    }

    fn previous(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => FeedSelection,
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            _ => *self,
        }
    }
}

pub struct App {
    pub state: AppState,

    pub config: Arc<Config>,
    pub news_flash_utils: Arc<NewsFlashUtils>,
    pub message_sender: UnboundedSender<Message>,

    pub tooltip: Tooltip<'static>,

    pub feed_list: FeedList,
    pub articles_list: ArticlesList,
    pub article_content: ArticleContent,
    pub command_line: CommandInput,
    pub async_operation_throbber: ThrobberState,

    pub is_running: bool,

    raw_input: Arc<Mutex<bool>>,
}

impl App {
    pub fn new(
        config: Config,
        news_flash_utils: NewsFlashUtils,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating new App instance");
        let config_arc = Arc::new(config);
        let news_flash_utils_arc = Arc::new(news_flash_utils);

        debug!("Initializing UI components");
        let app = Self {
            state: AppState::FeedSelection,
            config: Arc::clone(&config_arc),
            news_flash_utils: news_flash_utils_arc.clone(),
            is_running: true,
            message_sender: message_sender.clone(),
            feed_list: FeedList::new(
                Arc::clone(&config_arc),
                news_flash_utils_arc.clone(),
                message_sender.clone(),
            ),
            articles_list: ArticlesList::new(
                Arc::clone(&config_arc),
                news_flash_utils_arc.clone(),
                message_sender.clone(),
            ),
            article_content: ArticleContent::new(
                Arc::clone(&config_arc),
                news_flash_utils_arc.clone(),
                message_sender.clone(),
            ),
            command_line: CommandInput::new(
                Arc::clone(&config_arc),
                news_flash_utils_arc.clone(),
                message_sender.clone(),
            ),
            tooltip: Tooltip::new(
                "Welcome to eilmeldung".into(),
                crate::ui::tooltip::TooltipFlavor::Info,
            ),
            async_operation_throbber: ThrobberState::default(),
            raw_input: Arc::new(Mutex::new(false)),
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

        let config_arc = self.config.clone();
        let message_sender = self.message_sender.clone();
        debug!("Spawning input processing task");

        InputCommandHandler::spawn_input_command_handler(
            config_arc,
            message_sender,
            self.raw_input.clone(),
        );

        debug!("Sending ApplicationStarted command");
        self.message_sender
            .send(Message::Event(Event::ApplicationStarted))?;

        info!("Starting command processing loop");
        self.process_commands(message_receiver, terminal).await?;

        info!("Application run loop completed");
        Ok(())
    }

    fn tick(&mut self) {
        if self.news_flash_utils.is_async_operation_running() {
            trace!("Async operation running, updating throbber");
            self.async_operation_throbber.calc_next();
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

                            Message::SetRawInput(raw_input_value) => {
                                let mut raw_input = self.raw_input.lock().await;
                                *raw_input = *raw_input_value;
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

                        if let Err(e) = self.command_line.process_command(&message).await {
                            error!("Failed to process command line message: {}", e);
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
                self.news_flash_utils.sync_feeds();
            }

            Message::Event(Tooltip(tooltip)) => {
                trace!("Tooltip updated");
                self.tooltip = tooltip.clone();
            }

            Message::Event(Tick) => {
                self.tick();
            }

            Message::Event(AsyncOperationFailed(error, starting_event)) => {
                error!("Async operation {} failed: {:?}", error, starting_event);
                self.tooltip = crate::ui::tooltip::Tooltip::from_str(
                    error.clone().as_str(),
                    TooltipFlavor::Error,
                );
            }

            // TODO refactor redundant code!
            Message::Command(PanelFocus(next_state)) => {
                let old_state = self.state;
                self.state = *next_state;
                debug!("Focus moved from {:?} to {:?}", old_state, self.state);
                self.message_sender
                    .send(Message::Event(ApplicationStateChanged(self.state)))?;
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

            Message::Command(PanelFocusPreviousCyclic) => {
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

            _ => {}
        }

        Ok(())
    }
}
