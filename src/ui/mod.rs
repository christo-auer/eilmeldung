mod article_content;
mod articles_list;
mod batch;
mod command_confirm;
mod command_input;
mod feeds_list;
mod help_popup;
mod mouse;
mod state;
mod tooltip;
mod view;

pub mod prelude {
    pub use super::App;
    pub use super::article_content::prelude::*;
    pub use super::articles_list::prelude::*;
    pub use super::batch::BatchProcessor;
    pub use super::command_confirm::CommandConfirm;
    pub use super::command_input::CommandInput;
    pub use super::feeds_list::prelude::*;
    pub use super::help_popup::HelpPopup;
    pub use super::mouse::PanelAreas;
    pub use super::state::AppState;
    pub use super::tooltip::{Tooltip, TooltipFlavor, tooltip};
}

use crate::prelude::*;

use chrono::TimeDelta;
use log::{debug, error, info, trace, warn};
use news_flash::error::{FeedApiError, NewsFlashError};
use notify_rust::{Notification, Timeout};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{Event as TermEvent, MouseEventKind};
use ratatui_image::picker::Picker;
use std::collections::HashMap;
use std::process::Stdio;
use std::{path::Path, sync::Arc, time::Duration};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct App {
    state: AppState,

    config: Arc<Config>,
    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,
    config_file_manager: ConfigFileManager,

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

    panel_areas: PanelAreas,

    /// When true, the user is dragging the horizontal border; stores the initial row of the drag.
    drag_resize_active: bool,

    /// Override for the articles/content split height (absolute row count for articles list).
    articles_height_override: Option<u16>,
}

impl App {
    pub fn new(
        config: Config,
        config_file_manager: ConfigFileManager,
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
            config_file_manager,
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
            batch_processor: BatchProcessor::new(
                config_arc.clone(),
                news_flash_utils.clone(),
                message_sender.clone(),
            ),
            help_popup: HelpPopup::new(config_arc.clone(), message_sender.clone()),
            command_confirm: CommandConfirm::new(config_arc.clone(), message_sender.clone()),
            tooltip: Tooltip::new(
                "Stay up-to-date! Press `c e` to add eilmeldung release feed!".into(),
                crate::ui::tooltip::TooltipFlavor::Info,
            ),
            async_operation_throbber: ThrobberState::default(),
            is_offline: false,
            panel_areas: PanelAreas::default(),
            drag_resize_active: false,
            articles_height_override: None,
        };

        info!("App instance created with initial state: FeedSelection");
        app
    }

    pub async fn run(
        mut self,
        mut term_event_receiver: UnboundedReceiver<TermEvent>,
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
        debug!(
            "executing startup commands: {:?}",
            self.config.startup_commands
        );
        self.batch_processor.show_popup();
        self.message_sender
            .send(Message::Batch(self.config.startup_commands.to_vec()))?;

        info!("Starting command processing loop");
        self.process_messages(&mut term_event_receiver, &mut message_receiver, terminal)
            .await?;

        // close channels
        drop(term_event_receiver);
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

    async fn process_messages(
        mut self,
        term_event_receiver: &mut UnboundedReceiver<TermEvent>,
        message_receiver: &mut UnboundedReceiver<Message>,
        mut terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        let mut update_millis = 1000 / self.config.refresh_fps;
        let mut render_interval = tokio::time::interval(Duration::from_millis(update_millis));
        debug!(
            "Command processing loop started with {}fps refresh rate",
            self.config.refresh_fps
        );

        while self.is_running {
            // update FPS if value has changed
            let current_update_millis = 1000 / self.config.refresh_fps;
            if update_millis != current_update_millis {
                update_millis = current_update_millis;
                render_interval = tokio::time::interval(Duration::from_millis(update_millis));
            }

            let can_process_batch =
                !self.batch_processor.waiting_for_async_operation() && message_receiver.is_empty();

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

                term_event = term_event_receiver.recv(), if !self.batch_processor.has_commands() => {
                    // pass on term events until consumed
                    if let Some(term_event) = term_event.as_ref() {
                        self.process_term_event(term_event).await?
                            .pass_to(term_event, &mut self.help_popup).await?
                            .pass_to(term_event, &mut self.command_input).await?
                            .pass_to(term_event, &mut self.command_confirm).await?
                            .pass_to(term_event, &mut self.input_command_generator).await?;
                    }

                },


                message = message_receiver.recv() =>  {
                    if let Some(message) = message {

                        let mut redraw = matches!(message, Message::Command(Command::Redraw));

                        // we must handle Clear here as we have access to terminal
                        if matches!(message, Message::Command(Command::Clear)) {
                            if let Err(error) = terminal.clear() {
                                tooltip(&self.message_sender, &*error.to_string(), TooltipFlavor::Error)?;
                            }
                            redraw = true;
                        }

                        self.process_message(&message).await?;
                        self.input_command_generator.process_message(&message).await?;
                        self.config_file_manager.process_message(&message).await?;
                        self.batch_processor.process_message(&message).await?;
                        self.feed_list.process_message(&message).await?;
                        self.articles_list.process_message(&message).await?;
                        self.article_content.process_message(&message).await?;
                        self.command_input.process_message(&message).await?;
                        self.command_confirm.process_message(&message).await?;
                        self.help_popup.process_message(&message).await?;

                        if redraw {
                            terminal.draw(|frame| {
                                frame.render_widget(
                                    ratatui::widgets::Block::default()
                                      .style(Style::default().bg(*self.config.theme.color_palette().background())), frame.area()
                                );

                                frame.render_widget(&mut self, frame.area());

                            }

                                )?;
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
                    &*format!("unable to read OPML file: {error}"),
                    TooltipFlavor::Error,
                )?;
                return Ok(());
            }
        };

        self.news_flash_utils.import_opml(opml, false);

        Ok(())
    }

    async fn export_opml(&self, path_str: &str) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        let opml = news_flash.export_opml().await?;

        if let Err(error) = tokio::fs::write(Path::new(path_str), opml).await {
            tooltip(
                &self.message_sender,
                &*format!("unable to write OPML file: {error}"),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        }
        Ok(())
    }

    fn logout(&self) {
        self.news_flash_utils.logout();
    }

    async fn after_sync_notify(
        &self,
        new_articles: &HashMap<news_flash::models::FeedID, i64>,
    ) -> color_eyre::Result<()> {
        // show a tooltip
        let new_count = new_articles.values().sum::<i64>();
        tooltip(
            &self.message_sender,
            &*format!("{new_count} new articles synced"),
            TooltipFlavor::Info,
        )?;

        // don't do anything if no notification is wanted or needed
        if !self.config.notify_after_sync || new_articles.values().sum::<i64>() == 0 {
            return Ok(());
        }

        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        let output = self
            .config
            .notify_after_sync_stats_format
            .gen_output(&news_flash, new_articles)?;

        let Some((summary, body)) = output.split_once("\n") else {
            tooltip(
                &self.message_sender,
                "invalid format of sync stats",
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

        match self.config.notify_after_sync_cmd.as_ref() {
            None => self.notify_via_lib(summary, body)?,
            Some(command) => self.notify_via_command(command, summary, body)?,
        }

        Ok(())
    }

    fn notify_via_lib(&self, summary: &str, body: &str) -> color_eyre::Result<()> {
        let result = Notification::new()
            .summary(summary)
            .body(body)
            .icon("rss")
            .timeout(Timeout::default())
            .show();

        if let Err(error) = result {
            tooltip(
                &self.message_sender,
                &*format!(
                    "unable to send desktop notification while after_sync_notify=true: {error}"
                ),
                TooltipFlavor::Error,
            )?
        };

        Ok(())
    }

    fn notify_via_command(
        &self,
        command: &str,
        summary: &str,
        body: &str,
    ) -> color_eyre::Result<()> {
        let command = command
            .replace("{summary}", summary)
            .replace("{body}", body);

        let Ok((command, args)) = prepare_command(&command) else {
            tooltip(
                &self.message_sender,
                &*format!("invalid notify after sync cmd: {command}",),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

        let spawn = std::process::Command::new(&command)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn();

        let Ok(_) = spawn else {
            tooltip(
                &self.message_sender,
                &*format!("unable to execute {command}: {}", spawn.unwrap_err()),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

        Ok(())
    }

    async fn load_config(&mut self) -> color_eyre::Result<Option<Config>> {
        let load_config_res = self.config_file_manager.load_config().await;

        let Ok(config) = load_config_res else {
            tooltip(
                &self.message_sender,
                &*format!(
                    "could not read config file: {}",
                    load_config_res.unwrap_err()
                ),
                TooltipFlavor::Error,
            )?;
            return Ok(None);
        };

        Ok(Some(config))
    }

    async fn reload_config(&mut self) -> color_eyre::Result<()> {
        if let Ok(Some(mut config)) = self.load_config().await
            && self.validate_config(&mut config).await.is_ok()
        {
            tooltip(&self.message_sender, "config reloaded", TooltipFlavor::Info)?;

            self.message_sender
                .send(Message::Event(Event::ConfigReloaded(Arc::new(config))))?;
        }

        Ok(())
    }

    async fn validate_config(&mut self, config: &mut Config) -> color_eyre::Result<()> {
        if let Err(error) = self.config_file_manager.validate_config(config).await {
            tooltip(
                &self.message_sender,
                &*format!("config could not be reloaded: {error}"),
                TooltipFlavor::Error,
            )?;
            return Err(color_eyre::eyre::eyre!(
                "config could not be reloaded {error}"
            ));
        }
        Ok(())
    }

    async fn change_theme(&mut self, theme_name: Option<&String>) -> color_eyre::Result<()> {
        let theme = match theme_name {
            Some(theme_name) => {
                let Some(theme) = self
                    .config
                    .theme
                    .base16_themes()
                    .iter()
                    .find(|entry| entry.name() == *theme_name)
                    .cloned()
                else {
                    tooltip(
                        &self.message_sender,
                        &*format!("theme with name {theme_name} does not exist"),
                        TooltipFlavor::Error,
                    )?;
                    return Ok(());
                };
                Some(theme)
            }
            None => None,
        };

        if let Ok(Some(mut config)) = self.load_config().await {
            *config.theme.runtime_base16_theme_mut() = theme;

            let res = self.config_file_manager.validate_config(&mut config).await;
            match res {
                Ok(()) => {
                    self.message_sender
                        .send(Message::Event(Event::ConfigReloaded(Arc::new(config))))?;
                }
                Err(error) => {
                    tooltip(
                        &self.message_sender,
                        &*format!("config could be loaded: {error}"),
                        TooltipFlavor::Error,
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl TermEventHandler for App {
    async fn process_term_event(
        &mut self,
        event: &TermEvent,
    ) -> color_eyre::Result<TermEventForwarding> {
        match event {
            TermEvent::Mouse(mouse_event) if !matches!(mouse_event.kind, MouseEventKind::Moved) => {
                self.handle_mouse_event(mouse_event)?;
                Ok(TermEventForwarding::Consumed)
            }

            TermEvent::Resize(width, height) => {
                self.message_sender
                    .send(Message::Event(Event::Resized(*width, *height)))?;

                // silently ignore if querying picker fails
                if let Ok(picker) = Picker::from_query_stdio() {
                    self.message_sender
                        .send(Message::Event(Event::ImageProtocolPickerUpdated(picker)))?;
                }

                Ok(TermEventForwarding::Consumed)
            }

            _ => Ok(TermEventForwarding::PassOn),
        }
    }
}

impl MessageReceiver for App {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
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

            Message::Command(Undo) => {
                let undo_operation = self.news_flash_utils.undo_last_operation().await;

                match undo_operation {
                    Some(undo_operation) => tooltip(
                        &self.message_sender,
                        &*undo_operation.to_string(),
                        TooltipFlavor::Info,
                    )?,
                    None => {
                        let message = if self.news_flash_utils.is_async_operation_running() {
                            "nothing to undo yet (wait for async operation to finish)".to_string()
                        } else {
                            "nothing to undo".to_string()
                        };

                        tooltip(&self.message_sender, &*message, TooltipFlavor::Warning)?;
                    }
                }
            }

            Message::Command(Command::ChangeTheme(theme_name)) => {
                self.change_theme(theme_name.as_ref()).await?;
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
                        if matches!(
                            news_flash_error,
                            NewsFlashError::API(FeedApiError::Auth) | NewsFlashError::NotLoggedIn
                        ) {
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
                            // silently ignore thumbnail fetch errors
                        } else if !matches!(news_flash_error, NewsFlashError::Thumbnail) {
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

            Message::Command(ReloadConfig) => {
                self.reload_config().await?;
            }

            Message::Event(Event::ConfigFileChanged) if self.config.auto_reload_config => {
                self.reload_config().await?;
            }

            Message::Event(ConfigReloaded(config)) => {
                self.config = Arc::clone(config);
            }

            Message::Event(Event::ConnectionAvailable) => {
                let news_flash = self.news_flash_utils.news_flash_lock.read().await;

                if news_flash.is_offline() {
                    tooltip(
                        &self.message_sender,
                        "Trying to get online...",
                        TooltipFlavor::Info,
                    )?;
                    self.news_flash_utils.rebuild_client(&self.config).await?;
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
                                "Service is not reachable anymore, going offline",
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

            Message::Event(Event::AsyncSyncFinished(new_articles)) => {
                info!(
                    "scheduling after sync commands: {:?}",
                    self.config.after_sync_commands
                );
                self.batch_processor.show_popup();
                self.message_sender
                    .send(Message::Batch(self.config.after_sync_commands.to_vec()))?;

                self.after_sync_notify(new_articles).await?;
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
