use ratatui::DefaultTerminal;
use std::{sync::Arc, time::Duration};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    commands::{Command, CommandReceiver},
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AppState {
    FeedSelection,
    ArticleSelection,
    ArticleContent,
}

impl AppState {
    fn previous_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
            FeedSelection => ArticleContent,
        }
    }

    fn next_cyclic(&mut self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => FeedSelection,
        }
    }

    fn next(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => ArticleSelection,
            ArticleSelection => ArticleContent,
            ArticleContent => ArticleContent,
        }
    }

    fn previous(&self) -> AppState {
        use AppState::*;
        match self {
            FeedSelection => FeedSelection,
            ArticleSelection => FeedSelection,
            ArticleContent => ArticleSelection,
        }
    }
}

pub struct App {
    pub state: AppState,

    pub config: Arc<Config>,
    pub news_flash_async_manager: Arc<NewsFlashAsyncManager>,
    pub command_sender: UnboundedSender<Command>,

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
        command_sender: UnboundedSender<Command>,
    ) -> Self {
        let config_arc = Arc::new(config);
        let news_flash_async_manager_arc = Arc::new(news_flash_async_manager);
        Self {
            state: AppState::FeedSelection,
            config: Arc::clone(&config_arc),
            news_flash_async_manager: news_flash_async_manager_arc.clone(),
            is_running: true,
            command_sender: command_sender.clone(),
            feed_list: FeedList::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                command_sender.clone(),
            ),
            articles_list: ArticlesList::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                command_sender.clone(),
            ),
            article_content: ArticleContent::new(
                Arc::clone(&config_arc),
                news_flash_async_manager_arc.clone(),
                command_sender.clone(),
            ),
            tooltip: Tooltip::new(
                "Welcome to eilmeldung".into(),
                crate::ui::tooltip::TooltipFlavor::Info,
            ),
            async_operation_throbber: ThrobberState::default(),
        }
    }

    pub async fn run(
        mut self,
        command_receiver: UnboundedReceiver<Command>,
        terminal: DefaultTerminal,
    ) -> color_eyre::Result<()> {
        self.feed_list.build_tree().await?;

        let input_config = self.config.clone();
        let input_tx = self.command_sender.clone();
        tokio::spawn(async move { App::process_input(input_config.clone(), input_tx).await });

        self.command_sender.send(Command::ApplicationStarted)?;
        self.process_commands(command_receiver, terminal).await?;

        Ok(())
    }

    async fn process_input(
        config: Arc<Config>,
        tx: UnboundedSender<Command>,
    ) -> color_eyre::Result<()> {
        loop {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => {
                    let commands = translate_to_commands(&config.input_config, key_event);
                    for command in commands {
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
        let mut render_interval =
            tokio::time::interval(Duration::from_millis(1000 / self.config.refresh_fps));

        while self.is_running {
            tokio::select! {
                _ = render_interval.tick() => {
                    if self.news_flash_async_manager.is_async_operation_running() {
                        self.async_operation_throbber.calc_next();
                        terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
                    }
                }

                command = rx.recv() =>  {
                    if let Some(command) = command {
                        self.process_command(&command).await?;
                        self.feed_list.process_command(&command).await?;
                        self.articles_list.process_command(&command).await?;
                        self.article_content.process_command(&command).await?;
                    }
                    terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

                }

            }
        }

        Ok(())
    }
}

impl CommandReceiver for App {
    async fn process_command(&mut self, command: &Command) -> color_eyre::Result<()> {
        use Command::*;
        match command {
            ApplicationQuit => self.is_running = false,
            Sync => {
                self.news_flash_async_manager.sync_feeds();
            }

            Tooltip(tooltip) => self.tooltip = tooltip.clone(),

            AsyncSyncFinished(_)
            | AsyncFetchThumbnailFinished(_)
            | AsyncFetchFatArticleFinished(_)
            | AsyncMarkArticlesAsReadFinished => {
                self.tooltip = crate::ui::tooltip::Tooltip::new(
                    "async operation finished".to_string(),
                    TooltipFlavor::Info,
                );
            }

            AsyncSyncStarted
            | AsyncFetchThumbnailStarted
            | AsyncFetchFatArticleStarted
            | AsyncMarkArticlesAsReadStarted => {
                self.tooltip = crate::ui::tooltip::Tooltip::new(
                    "async operation started".to_string(),
                    TooltipFlavor::Info,
                );
            }

            AsyncOperationFailed(error) => {
                self.tooltip =
                    crate::ui::tooltip::Tooltip::new(error.clone(), TooltipFlavor::Error);
            }

            FocusNext => {
                self.state = self.state.next();
                self.command_sender
                    .send(Command::ApplicationStateChanged(self.state))?;
            }

            FocusPrevious => {
                self.state = self.state.previous();
                self.command_sender
                    .send(Command::ApplicationStateChanged(self.state))?;
            }

            CyclicFocusNext => {
                self.state = self.state.next_cyclic();
                self.command_sender
                    .send(Command::ApplicationStateChanged(self.state))?;
            }

            CyclicFocusPrevious => {
                self.state = self.state.previous_cyclic();
                self.command_sender
                    .send(Command::ApplicationStateChanged(self.state))?;
            }

            _ => {}
        }

        Ok(())
    }
}
