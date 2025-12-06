use crate::prelude::*;

use log::{debug, error, info, trace};
use ratatui::DefaultTerminal;
use std::{fmt::Display, str::FromStr, sync::Arc, time::Duration};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum AppState {
    #[default]
    FeedSelection,
    ArticleSelection,
    ArticleContent,
    ArticleContentDistractionFree,
}

impl From<Panel> for AppState {
    fn from(value: Panel) -> Self {
        match value {
            Panel::FeedList => Self::FeedSelection,
            Panel::ArticleList => Self::ArticleSelection,
            Panel::ArticleContent => Self::ArticleContent,
        }
    }
}

impl FromStr for AppState {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "feeds" => Self::FeedSelection,
            "articles" => Self::ArticleSelection,
            "content" => Self::ArticleContent,
            "zen" => Self::ArticleContentDistractionFree,
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "expected feeds, articles, content or zen"
                ));
            }
        })
    }
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
    fn previous_cyclic(&self) -> AppState {
        use AppState::*;
        match self {
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            FeedSelection => ArticleContent,
            _ => *self,
        }
    }

    fn next_cyclic(&self) -> AppState {
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

    pub input_command_generator: InputCommandGenerator,
    pub feed_list: FeedList,
    pub articles_list: ArticlesList,
    pub article_content: ArticleContent,
    pub command_input: CommandInput,
    pub command_confirm: CommandConfirm,
    pub help_popup: HelpPopup<'static>,
    pub async_operation_throbber: ThrobberState,

    pub is_offline: bool,

    pub is_running: bool,
}

impl App {
    pub fn new(
        config: Config,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating new App instance");
        let config_arc = Arc::new(config);

        debug!("Initializing UI components");
        let app = Self {
            state: AppState::FeedSelection,
            config: Arc::clone(&config_arc),
            news_flash_utils: news_flash_utils.clone(),
            is_running: true,
            message_sender: message_sender.clone(),
            input_command_generator: InputCommandGenerator::new(
                config_arc.clone(),
                message_sender.clone(),
            ),
            feed_list: FeedList::new(
                config_arc.clone(),
                news_flash_utils.clone(),
                message_sender.clone(),
            ),
            articles_list: ArticlesList::new(
                config_arc.clone(),
                news_flash_utils.clone(),
                message_sender.clone(),
            ),
            article_content: ArticleContent::new(
                config_arc.clone(),
                news_flash_utils.clone(),
                message_sender.clone(),
            ),
            command_input: CommandInput::new(
                config_arc.clone(),
                news_flash_utils.clone(),
                message_sender.clone(),
            ),
            help_popup: HelpPopup::new(config_arc.clone()),
            command_confirm: CommandConfirm::new(config_arc.clone(), message_sender.clone()),
            tooltip: Tooltip::new(
                "Welcome to eilmeldung".into(),
                crate::ui::tooltip::TooltipFlavor::Info,
            ),
            async_operation_throbber: ThrobberState::default(),
            is_offline: false,
        };

        info!("App instance created with initial state: FeedSelection");
        app
    }

    pub async fn run(
        mut self,
        mut message_receiver: UnboundedReceiver<Message>,
        terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        info!("Starting application run loop");

        debug!("get offline state");
        self.is_offline = self
            .news_flash_utils
            .news_flash_lock
            .read()
            .await
            .is_offline();

        debug!("Sending ApplicationStarted command");
        self.message_sender
            .send(Message::Event(Event::ApplicationStarted))?;
        debug!("Select feeds panel");
        self.message_sender
            .send(Message::Command(Command::PanelFocus(Panel::FeedList)))?;

        info!("Starting command processing loop");
        self.process_commands(&mut message_receiver, terminal)
            .await?;

        // closing receiver
        drop(message_receiver);

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
        rx: &mut UnboundedReceiver<Message>,
        mut terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        let mut render_interval =
            tokio::time::interval(Duration::from_millis(1000 / self.config.refresh_fps));
        debug!(
            "Command processing loop started with {}fps refresh rate",
            self.config.refresh_fps
        );

        let mut redraw = true;

        while self.is_running {
            tokio::select! {
                _ = render_interval.tick() => {
                    self.message_sender.send(Message::Event(Event::Tick))?;
                }

                message = rx.recv() =>  {
                    if let Some(message) = message {
                        match &message {
                            Message::Event(Event::Tick) => {
                                if self.news_flash_utils.is_async_operation_running(){
                                    redraw = true;
                                }
                            },

                            Message::Event(Event::Resized(_width, _height)) => {
                                redraw = true;
                            }

                            Message::Event(Event::AsyncArticleThumbnailFetchFinished(_)) => {
                                trace!("Processing message: AsyncFetchThumbnailFinished");
                            }

                            _ => {
                                redraw = true;
                                trace!("Processing message: {:?}", message);
                            }

                        }


                        if !self.command_input.is_active() && !self.command_confirm.is_active() {
                            self.input_command_generator.process_command(&message).await?;
                        }

                        self.process_command(&message).await?;
                        self.feed_list.process_command(&message).await?;
                        self.articles_list.process_command(&message).await?;
                        self.article_content.process_command(&message).await?;
                        self.command_input.process_command(&message).await?;
                        self.command_confirm.process_command(&message).await?;
                        self.help_popup.process_command(&message).await?;

                    } else {
                        debug!("Message channel closed, stopping message processing");
                        break;
                    }

                    if redraw {
                        redraw = false;
                        if let Err(e) = terminal.draw(|frame| frame.render_widget(&mut self, frame.area())) {
                            error!("Failed to render terminal: {}", e);
                        }
                    }
                }
            }
        }

        info!("Message processing loop ended");
        Ok(())
    }

    fn switch_state(&mut self, next_state: AppState) -> color_eyre::eyre::Result<()> {
        let old_state = self.state;
        self.state = next_state;
        debug!("Focus moved from {:?} to {:?}", old_state, self.state);
        self.message_sender
            .send(Message::Event(Event::ApplicationStateChanged(self.state)))?;

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

            Message::Event(Tooltip(tooltip)) => {
                trace!("Tooltip updated");
                self.tooltip = tooltip.clone();
            }

            Message::Event(Tick) => {
                self.tick();
            }

            Message::Event(AsyncOperationFailed(error, starting_event)) => {
                error!("Async operation {} failed: {:?}", error, starting_event);

                match error {
                    AsyncOperationError::NewsFlashError(news_flash_error) => {
                        tooltip(
                            &self.message_sender,
                            NewsFlashUtils::error_to_message(news_flash_error).as_str(),
                            TooltipFlavor::Error,
                        )?;
                    }
                    AsyncOperationError::Report(report) => {
                        tooltip(
                            &self.message_sender,
                            report.to_string().as_str(),
                            TooltipFlavor::Error,
                        )?;
                    }
                }
            }

            Message::Command(PanelFocus(next_state)) => {
                self.switch_state((*next_state).into())?;
            }

            Message::Command(PanelFocusNext) => {
                self.switch_state(self.state.next())?;
            }

            Message::Command(PanelFocusPrevious) => {
                self.switch_state(self.state.previous())?;
            }

            Message::Command(PanelFocusNextCyclic) => {
                self.switch_state(self.state.next_cyclic())?;
            }

            Message::Command(PanelFocusPreviousCyclic) => {
                self.switch_state(self.state.previous_cyclic())?;
            }

            Message::Event(Event::ConnectionAvailable) => {
                let news_flash = self.news_flash_utils.news_flash_lock.read().await;

                if news_flash.is_offline() {
                    tooltip(
                        &self.message_sender,
                        "Trying to get online...",
                        TooltipFlavor::Info,
                    )?;
                    self.news_flash_utils.set_offline(false);
                }
            }

            Message::Event(Event::ConnectionLost(reason)) => {
                if !self.is_offline {
                    match reason {
                        ConnectionLostReason::NoInternet => {
                            tooltip(
                                &self.message_sender,
                                "Connection to internet lost, going offline",
                                TooltipFlavor::Warning,
                            )?;
                        }
                        ConnectionLostReason::NotReachable => {
                            tooltip(
                                &self.message_sender,
                                "Service is not reachable any more, going offline",
                                TooltipFlavor::Warning,
                            )?;
                        }
                    }
                    self.news_flash_utils.set_offline(true);
                }
            }

            Message::Event(Event::AsyncSetOfflineFinished(offline)) => {
                info!("new offline state: {}", offline);
                self.is_offline = *offline;

                if !offline {
                    tooltip(&self.message_sender, "Online again", TooltipFlavor::Info)?;
                }
            }

            Message::Command(ToggleDistractionFreeMode) => {
                let old_state = self.state;
                let new_state = match old_state {
                    AppState::ArticleContentDistractionFree => AppState::ArticleContent,
                    _ => AppState::ArticleContentDistractionFree,
                };
                self.switch_state(new_state)?;
            }

            _ => {}
        }

        Ok(())
    }
}
