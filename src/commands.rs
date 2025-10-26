use std::{collections::HashMap, fmt::Display, str::FromStr};

use news_flash::models::{Article, FatArticle, FeedID, Thumbnail};
use ratatui::crossterm::event::KeyEvent;

use crate::{
    app::AppState,
    query::AugmentedArticleFilter,
    ui::{articles_list::ArticleScope, tooltip::Tooltip},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    // general navigation
    NavigateUp,
    NavigateDown,
    NavigatePageUp,
    NavigatePageDown,
    NavigateFirst,
    NavigateLast,
    NavigateLeft,
    NavigateRight,

    // Panels
    PanelFocusNext,
    PanelFocus(AppState),
    PanelFocusPrevious,
    PanelFocusNextCyclic,
    PanelFocusPreviousCyclic,
    ToggleDistractionFreeMode,

    // feeds and articles
    FeedsSync,
    ArticleCurrentOpenInBrowser,
    ArticleCurrentSetRead,
    ArticleCurrentSetUnread,
    ArticleCurrentToggleRead,
    ArticleListSelectNextUnread,
    ArticleListSetAllRead,
    ArticleListSetAllUnread,
    ArticleListSetScope(ArticleScope),
    ArticleCurrentScrape,

    // application
    ApplicationQuit,

    // command line
    CommandLineOpen(String),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        match self.clone() {
            NavigateUp => write!(f, "up"),
            NavigateDown => write!(f, "down"),
            NavigatePageUp => write!(f, "page up"),
            NavigatePageDown => write!(f, "page down"),
            NavigateFirst => write!(f, "to first"),
            NavigateLast => write!(f, "to last"),
            NavigateLeft => write!(f, "left"),
            NavigateRight => write!(f, "right"),
            PanelFocusNext => write!(f, "focus next"),
            PanelFocus(app_state) => write!(f, "focus {}", app_state),
            PanelFocusPrevious => write!(f, "focus previous"),
            PanelFocusNextCyclic => write!(f, "focus next"),
            PanelFocusPreviousCyclic => write!(f, "focus next"),
            ToggleDistractionFreeMode => write!(f, "distraction free mode"),
            FeedsSync => write!(f, ""),
            ArticleCurrentOpenInBrowser => write!(f, "open in browser"),
            ArticleCurrentSetRead => write!(f, "mark read"),
            ArticleCurrentSetUnread => write!(f, "mark unread"),
            ArticleCurrentToggleRead => write!(f, "toggel read"),
            ArticleListSelectNextUnread => write!(f, "select next unread"),
            ArticleListSetAllRead => write!(f, "mark all read"),
            ArticleListSetAllUnread => write!(f, "mark all unread"),
            ArticleListSetScope(ArticleScope::Marked) => write!(f, "show marked"),
            ArticleListSetScope(ArticleScope::Unread) => write!(f, "show unread"),
            ArticleListSetScope(ArticleScope::All) => write!(f, "show all"),
            ArticleCurrentScrape => write!(f, "scrape content"),
            ApplicationQuit => write!(f, "quit"),
            CommandLineOpen(input) => write!(f, "command line {}", input),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct CommandSequence {
    pub commands: Vec<Command>,
}

impl From<Command> for CommandSequence {
    fn from(single_command: Command) -> Self {
        Self {
            commands: vec![single_command],
        }
    }
}

impl From<Vec<Command>> for CommandSequence {
    fn from(commands: Vec<Command>) -> Self {
        Self { commands }
    }
}

impl Display for CommandSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for command in self.commands.iter() {
            if !first {
                f.write_str(",")?;
            }
            command.fmt(f)?;

            first = false;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    ArticlesSelected(AugmentedArticleFilter),
    ArticleSelected(Article),
    FatArticleSelected(Article),

    AsyncSyncStarted,
    AsyncSyncFinished(HashMap<FeedID, i64>),

    AsyncFetchThumbnailStarted,
    AsyncFetchThumbnailFinished(Option<Thumbnail>),

    AsyncFetchFatArticleStarted,
    AsyncFetchFatArticleFinished(FatArticle),

    AsyncMarkArticlesAsReadStarted,
    AsyncMarkArticlesAsReadFinished,

    AsyncOperationFailed(String),

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),
}

#[derive(Debug)]
pub enum Message {
    Command(Command),
    Event(Event),
    SetRawInput(bool),
}

pub trait MessageReceiver {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()>;
}
