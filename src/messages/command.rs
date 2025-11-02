use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;

use crate::prelude::*;

#[derive(Clone, Debug)]
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

    // searching
    ArticleListSearch(ArticleQuery),
    ArticleListSearchNext,
    ArticleListSearchPrevious,
    ArticleListFilterSet(ArticleQuery),
    ArticleListFilterApply,
    ArticleListFilterClear,

    // application
    ApplicationQuit,

    // command line
    CommandLineOpen(Option<String>),
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
            FeedsSync => write!(f, "sync feeds"),
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
            CommandLineOpen(input) => write!(f, "command line {}", input.unwrap_or_default()),
            ArticleListSearch(query) => write!(f, "search article {}", query.query_string()),
            ArticleListSearchNext => write!(f, "article search next"),
            ArticleListSearchPrevious => write!(f, "article search previous"),
            ArticleListFilterSet(query) => {
                write!(f, "filter article list {}", query.query_string())
            }
            ArticleListFilterApply => write!(f, "apply current article filter"),
            ArticleListFilterClear => write!(f, "clear article filter"),
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

impl FromStr for Command {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed_command = s.trim();
        let command_end_pos = trimmed_command.find(" ");

        let command = match command_end_pos {
            Some(pos) => &trimmed_command[..pos],
            None => trimmed_command,
        };

        let args = command_end_pos.map(|pos| &trimmed_command[pos + 1..]);

        use Command::*;
        Ok(match command {
            "up" => NavigateUp,
            "down" => NavigateDown,
            "page_up" => NavigatePageUp,
            "page_down" => NavigatePageDown,
            "goto_first" => NavigateFirst,
            "goto_last" => NavigateLast,
            "left" => NavigateLeft,
            "right" => NavigateRight,

            "next" => PanelFocusNext,
            "focus" => match args {
                Some(args) => PanelFocus(AppState::from_str(args)?),
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "expected panel: feeds, articles, content or zen"
                    ));
                }
            },
            "prev" => PanelFocusPrevious,
            "nextc" => PanelFocusNextCyclic,
            "prevc" => PanelFocusPreviousCyclic,
            "zen" => ToggleDistractionFreeMode,

            "sync" => FeedsSync,
            "open" => ArticleCurrentOpenInBrowser,
            "read" => ArticleCurrentSetRead,
            "unread" => ArticleCurrentSetUnread,
            "toggle" => ArticleCurrentToggleRead,
            "next_unread" => ArticleListSelectNextUnread,
            "all_read" => ArticleListSetAllRead,
            "all_unread" => ArticleListSetAllUnread,
            "scope" => match args {
                Some(args) => ArticleListSetScope(ArticleScope::from_str(args)?),
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "expected scope: all, unread or marked"
                    ));
                }
            },
            "scrape" => ArticleCurrentScrape,

            "quit" | "q" => ApplicationQuit,

            "/" => match args {
                Some(args) => ArticleListSearch(ArticleQuery::from_str(args)?),
                None => return Err(color_eyre::eyre::eyre!("Search query expected")),
            },
            "/next" => ArticleListSearchNext,
            "/prev" => ArticleListSearchPrevious,

            "=" => match args {
                Some(args) => ArticleListFilterSet(ArticleQuery::from_str(args)?),
                None => return Err(color_eyre::eyre::eyre!("Search query expected")),
            },

            "=clear" => ArticleListFilterClear,

            "=apply" => ArticleListFilterApply,

            _ => {
                return Err(color_eyre::eyre::eyre!("Invalid command: {}", command));
            }
        })
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Command::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
