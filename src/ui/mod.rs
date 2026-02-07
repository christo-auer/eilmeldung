mod article_content;
mod articles_list;
mod batch;
mod command_confirm;
mod command_input;
mod feeds_list;
mod help_popup;
mod tooltip;
mod view;

pub mod prelude {
    pub use super::article_content::prelude::*;
    pub use super::articles_list::prelude::*;
    pub use super::batch::BatchProcessor;
    pub use super::command_confirm::CommandConfirm;
    pub use super::command_input::CommandInput;
    pub use super::feeds_list::prelude::*;
    pub use super::help_popup::HelpPopup;
    pub use super::tooltip::{Tooltip, TooltipFlavor, tooltip};
    pub use super::{App, AppState};
}

use crate::prelude::*;

use chrono::TimeDelta;
use log::{debug, error, info, trace, warn};
use news_flash::error::{FeedApiError, NewsFlashError};
use ratatui::DefaultTerminal;
use std::{fmt::Display, path::Path, str::FromStr, sync::Arc, time::Duration};
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
    state: AppState,

    config: Arc<Config>,
    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    tooltip: Tooltip<'static>,

    input_command_generator: InputCommandGenerator,
    feed_list: FeedList,
    articles_list: ArticlesList,
    article_content: ArticleContent,
    command_input: CommandInput,
    command_confirm: CommandConfirm,
    help_popup: HelpPopup<'static>,
    async_operation_throbber: ThrobberState,
    batch_processor: BatchProcessor,

    is_offline: bool,

    is_running: bool,
}

impl App {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating new App instance");
        let config_arc = config.clone();

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
            batch_processor: BatchProcessor::new(config_arc.clone(), message_sender.clone()),
            help_popup: HelpPopup::new(config_arc.clone(), message_sender.clone()),
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

        // set days before articles get removed
        info!(
            "setting amount of days before articles are removed to {}",
            self.config.keep_articles_days
        );
        self.news_flash_utils
            .news_flash_lock
            .read()
            .await
            .set_keep_articles_duration(Some(TimeDelta::days(
                self.config.keep_articles_days as i64,
            )))
            .await?;

        debug!("Sending ApplicationStarted command");
        self.message_sender
            .send(Message::Event(Event::ApplicationStarted))?;

        debug!("Select feeds panel");
        self.message_sender
            .send(Message::Command(Command::PanelFocus(Panel::FeedList)))?;

        // execute all startup commands
        debug!("executing startup commands");
        self.message_sender
            .send(Message::Batch(self.config.startup_commands.to_vec()))?;

        info!("Starting command processing loop");
        self.process_commands(&mut message_receiver, terminal)
            .await?;

        // closing receiver
        drop(message_receiver);

        info!("Application run loop completed");
        Ok(())
    }

    fn tick(&mut self) -> bool {
        if self.news_flash_utils.is_async_operation_running() {
            trace!("Async operation running, updating throbber");
            self.async_operation_throbber.calc_next();
            return true;
        }
        false
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

        while self.is_running {
            let can_process_batch =
                !self.news_flash_utils.is_async_operation_running() && rx.is_empty();

            tokio::select! {

                batch_command = self.batch_processor.next(), if can_process_batch => {
                    if let Ok(batch_command) = batch_command {
                        info!("sending next batch command {batch_command:?}");
                        self.message_sender.send(Message::Command(batch_command.to_owned()))?;
                    }
                }

                _ = render_interval.tick() => {
                    self.message_sender.send(Message::Event(Event::Tick))?;
                }


                message = rx.recv() =>  {
                    if let Some(message) = message {

                        // TODO refactor all this
                        if !self.batch_processor.has_commands()
                        && !self.command_input.is_active()
                        && !self.command_confirm.is_active()
                        && !self.help_popup.is_modal().unwrap_or(false)
                        {
                            self.input_command_generator.process_command(&message).await?;
                        }

                        self.batch_processor.process_command(&message).await?;
                        self.process_command(&message).await?;
                        self.feed_list.process_command(&message).await?;
                        self.articles_list.process_command(&message).await?;
                        self.article_content.process_command(&message).await?;
                        self.command_input.process_command(&message).await?;
                        self.command_confirm.process_command(&message).await?;
                        self.help_popup.process_command(&message).await?;

                        if matches!(message, Message::Command(Command::Redraw)) {
                            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
                        }

                    } else {
                        debug!("Message channel closed, stopping message processing");
                        break;
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

    async fn import_opml(&self, path_str: &str) -> color_eyre::Result<()> {
        let opml = match tokio::fs::read_to_string(Path::new(path_str)).await {
            Ok(opml) => opml,
            Err(error) => {
                tooltip(
                    &self.message_sender,
                    &*format!("Unable to read OPML file: {error}"),
                    TooltipFlavor::Error,
                )?;
                return Ok(());
            }
        };

        self.news_flash_utils.import_opml(opml, true);

        Ok(())
    }

    async fn export_opml(&self, path_str: &str) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        let opml = news_flash.export_opml().await?;

        if let Err(error) = tokio::fs::write(Path::new(path_str), opml).await {
            tooltip(
                &self.message_sender,
                &*format!("Unable to write OPML file: {error}"),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        }
        Ok(())
    }

    fn logout(&self) {
        self.news_flash_utils.logout();
    }
}

impl MessageReceiver for App {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;
        let mut needs_redraw = true;
        match message {
            Message::Command(Logout(confirmation)) => {
                if confirmation.as_str() != "NOW" {
                    tooltip(
                        &self.message_sender,
                        "not logging out, expected parameter `NOW` for confirmation",
                        TooltipFlavor::Warning,
                    )?;
                } else {
                    self.logout();
                }
            }

            Message::Command(ApplicationQuit) => {
                info!("Application quit requested");
                self.is_running = false;
            }

            Message::Command(ImportOpml(path_str)) => {
                self.import_opml(path_str).await?;
            }

            Message::Event(Event::AsyncLogoutFinished) => {
                self.message_sender
                    .send(Message::Command(Command::ApplicationQuit))?;
            }

            Message::Command(ExportOpml(path_str)) => {
                self.export_opml(path_str).await?;
            }

            Message::Event(Tooltip(tooltip)) => {
                trace!("Tooltip updated");
                self.tooltip = tooltip.clone();
                needs_redraw = true;
            }

            Message::Event(Resized(..)) => {
                trace!("terminal resized, forcing redraw");
                self.message_sender
                    .send(Message::Command(Command::Redraw))?;
            }

            Message::Event(Tick) => {
                needs_redraw = self.tick();
            }

            Message::Event(Event::AsyncImportOpmlFinished) => {
                tooltip(
                    &self.message_sender,
                    "OPML imported --- you should sync now",
                    TooltipFlavor::Info,
                )?;
            }

            Message::Event(AsyncOperationFailed(error, starting_event)) => {
                error!("Async operation {} failed: {:?}", error, starting_event);

                // abort any batch operations
                self.batch_processor.abort();

                match error {
                    AsyncOperationError::NewsFlashError(news_flash_error) => {
                        // Check if this is an auth error - if so, try to re-login and retry
                        if matches!(news_flash_error, NewsFlashError::API(FeedApiError::Auth)) {
                            warn!("Auth error detected, attempting re-login");
                            if self.news_flash_utils.relogin().await {
                                // Re-login succeeded, retry parameterless operations automatically
                                let retried = match starting_event.as_ref() {
                                    Event::AsyncSync => {
                                        info!("Retrying sync after re-login");
                                        self.news_flash_utils.sync();
                                        true
                                    }
                                    Event::AsyncSetAllRead => {
                                        info!("Retrying set_all_read after re-login");
                                        self.news_flash_utils.set_all_read();
                                        true
                                    }
                                    Event::AsyncLogout => {
                                        info!("Retrying logout after re-login");
                                        self.news_flash_utils.logout();
                                        true
                                    }
                                    _ => false,
                                };

                                if retried {
                                    tooltip(
                                        &self.message_sender,
                                        "Session expired, re-logged in and retrying...",
                                        TooltipFlavor::Info,
                                    )?;
                                } else {
                                    // For operations with parameters, just notify the user
                                    tooltip(
                                        &self.message_sender,
                                        "Session expired and refreshed. Please try again.",
                                        TooltipFlavor::Warning,
                                    )?;
                                }
                            } else {
                                // Re-login failed
                                tooltip(
                                    &self.message_sender,
                                    "Session expired and re-login failed. Please restart the app.",
                                    TooltipFlavor::Error,
                                )?;
                            }
                        } else {
                            tooltip(
                                &self.message_sender,
                                NewsFlashUtils::error_to_message(news_flash_error).as_str(),
                                TooltipFlavor::Error,
                            )?;
                        }
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
                    self.news_flash_utils.rebuild_client().await?;
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

            Message::Event(Event::AsyncSyncFinished(..)) => {
                info!(
                    "scheduling after sync commands: {:?}",
                    self.config.after_sync_commands
                );
                self.batch_processor.show_popup();
                self.message_sender
                    .send(Message::Batch(self.config.after_sync_commands.to_vec()))?;
            }

            _ => {
                needs_redraw = false;
            }
        }

        if needs_redraw {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
