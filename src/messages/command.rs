use std::fmt::Display;
use std::str::FromStr;

use news_flash::models::{Marked, Url};
use ratatui::style::Color;
use serde::Deserialize;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum ActionScope {
    Current,
    All,
    Query(ArticleQuery),
}

impl FromStr for ActionScope {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ActionScope::*;
        match s {
            "." => Ok(Current),
            "%" => Ok(All),
            _ => Ok(Query(ArticleQuery::from_str(s)?)),
        }
    }
}

impl ActionScope {
    fn from_option_string(s: Option<&str>) -> color_eyre::Result<ActionScope> {
        match s {
            Some(s) => ActionScope::from_str(s),
            None => Ok(ActionScope::Current),
        }
    }
}

impl Display for ActionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActionScope::*;
        match self {
            Current => write!(f, "current article")?,
            All => write!(f, "all articles")?,
            Query(query) => write!(f, "all articles matching {}", query.query_string())?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ActionSetReadTarget {
    Current,
    FeedList,
    ArticleList,
}

impl FromStr for ActionSetReadTarget {
    type Err = color_eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ActionSetReadTarget::*;

        Ok(match s {
            "." => Current,
            "feeds" => FeedList,
            "articles" => ArticleList,
            _ => return Err(color_eyre::eyre::eyre!("unknown target {s}")),
        })
    }
}

impl Display for ActionSetReadTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActionSetReadTarget::*;
        match self {
            Current => f.write_str("current panel"),
            FeedList => f.write_str("feed list"),
            ArticleList => f.write_str("article list"),
        }
    }
}

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

    // feed list
    FeedListSync,
    FeedListAddCategory(String),
    FeedListAddFeed(String, Url),
    FeedListRenameEntity(String),
    FeedListRemoveEntity,
    FeedListRemoveEntityWithChildren,
    FeedListFeedChangeUrl(Url),
    FeedListHighlightEntity,
    FeedListMoveHighlightedEntityHere,

    ActionSetRead(ActionSetReadTarget, ActionScope),
    ActionSetUnread(ActionScope),
    ActionSetMarked(ActionScope, Marked),
    ActionOpenInBrowser(ActionScope),

    ActionTagArticles(ActionScope, String),
    ActionUntagArticles(ActionScope, String),

    TagAdd(String, Option<Color>),
    TagRemove(String),
    TagRename(String, String),
    TagChangeColor(String, Color),

    // article list commands
    ArticleListSelectNextUnread,
    ArticleListSetScope(ArticleScope),
    ArticleCurrentScrape,

    // article list searching
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
            FeedListAddCategory(name) => write!(f, "add category {name}"),
            FeedListAddFeed(name, url) => write!(f, "add feed {name} with url {url}"),
            FeedListRenameEntity(name) => write!(f, "rename selected to {name}"),
            FeedListRemoveEntity => write!(f, "remove selected"),
            FeedListRemoveEntityWithChildren => write!(f, "remove selected and its children"),
            FeedListFeedChangeUrl(url) => write!(f, "change url of selected feed to {url}"),
            FeedListHighlightEntity => write!(f, "highlight entity"),
            FeedListMoveHighlightedEntityHere => write!(f, "move highlighted entity here"),
            ArticleListSelectNextUnread => write!(f, "select next unread"),
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

            FeedListSync => write!(f, "sync all"),
            ActionSetRead(target, action_scope) => {
                write!(f, "mark {action_scope} as read in {target}",)
            }
            ActionSetUnread(action_scope) => write!(f, "mark {} as unread", action_scope),
            ActionSetMarked(action_scope, Marked::Marked) => write!(f, "mark {}", action_scope),
            ActionSetMarked(action_scope, Marked::Unmarked) => write!(f, "unmark {}", action_scope),
            ActionOpenInBrowser(action_scope) => write!(f, "open {} in browser", action_scope),
            ActionTagArticles(action_scope, tag) => {
                write!(f, "add #{} to {}", tag, &action_scope)
            }
            ActionUntagArticles(action_scope, tag) => {
                write!(f, "remove #{} from {}", tag, &action_scope)
            }
            TagAdd(tag_title, _) => {
                write!(f, "add tag #{}", tag_title)
            }
            TagRemove(tag_title) => {
                write!(f, "remove tag {}", tag_title)
            }
            TagRename(tag_title, new_tag_title) => {
                write!(f, "rename #{} to #{}", tag_title, new_tag_title)
            }
            TagChangeColor(tag_title, color) => {
                write!(f, "change color of #{} to #{}", tag_title, color)
            }
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

fn split_off_first(s: &str) -> (String, Option<String>) {
    let trimmed = s.trim();
    let end_pos = trimmed.find(" ");

    let first = match end_pos {
        Some(pos) => &trimmed[..pos],
        None => trimmed,
    };

    let args = end_pos
        .map(|pos| (trimmed[pos + 1..]).to_owned())
        .to_owned();

    (first.to_owned(), args)
}

fn expect_word(s: &mut Option<String>, to_expect: &str) -> color_eyre::Result<String> {
    let Some(args) = s.as_mut() else {
        return Err(color_eyre::eyre::eyre!("expected {}", to_expect));
    };

    let (word, tail) = split_off_first(args.as_str());

    *s = tail;

    Ok(word)
}

fn expect_color(s: &mut Option<String>) -> color_eyre::Result<Color> {
    let word = expect_word(s, "color")?;
    Ok(Color::from_str(word.as_str())?)
}

fn expect_nothing(s: Option<String>) -> color_eyre::Result<()> {
    match s {
        Some(s) => Err(color_eyre::eyre::eyre!("unknown trailing arguments: {s}")),
        None => Ok(()),
    }
}

impl FromStr for Command {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = split_off_first(s);
        let mut args = split.1;
        let command = split.0.as_str();

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
                Some(args) => PanelFocus(AppState::from_str(args.as_str())?),
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

            "syncall" => FeedListSync,

            "read" => {
                let old_args = args.clone();
                let word = expect_word(&mut args, "target or scope");

                let target = word
                    .map(|word| match ActionSetReadTarget::from_str(word.as_str()) {
                        Ok(target) => target,
                        Err(_) => {
                            args = old_args; // restore old version of arggs
                            ActionSetReadTarget::Current
                        }
                    })
                    .unwrap_or(ActionSetReadTarget::Current);

                ActionSetRead(target, ActionScope::from_option_string(args.as_deref())?)
            }

            "unread" => ActionSetUnread(ActionScope::from_option_string(args.as_deref())?),
            "open" => ActionOpenInBrowser(ActionScope::from_option_string(args.as_deref())?),
            "mark" => ActionSetMarked(
                ActionScope::from_option_string(args.as_deref())?,
                Marked::Marked,
            ),
            "unmark" => ActionSetMarked(
                ActionScope::from_option_string(args.as_deref())?,
                Marked::Unmarked,
            ),
            tag_command @ ("tag" | "untag") => {
                let tag = expect_word(&mut args, "tag name")?;

                match tag_command {
                    "tag" => {
                        ActionTagArticles(ActionScope::from_option_string(args.as_deref())?, tag)
                    }
                    "untag" => {
                        ActionUntagArticles(ActionScope::from_option_string(args.as_deref())?, tag)
                    }
                    _ => unreachable!(),
                }
            }

            "tagchangecolor" => {
                let tag_title = expect_word(&mut args, "tag name")?;
                let color = expect_color(&mut args)?;
                expect_nothing(args)?;
                TagChangeColor(tag_title, color)
            }

            "tagadd" => {
                let tag_title = expect_word(&mut args, "tag name")?;
                let color = match args {
                    None => None,
                    _ => Some(expect_color(&mut args)?),
                };
                expect_nothing(args)?;
                TagAdd(tag_title, color)
            }

            "tagrename" => {
                let tag_title = expect_word(&mut args, "old tag name")?;
                let new_tag_title = expect_word(&mut args, "new tag name")?;
                expect_nothing(args)?;
                TagRename(tag_title, new_tag_title)
            }

            "tagremove" => {
                let tag_title = expect_word(&mut args, "tag name")?;
                expect_nothing(args)?;
                TagRemove(tag_title)
            }

            "nextu" | "nextunread" => ArticleListSelectNextUnread,

            "scope" => match args {
                Some(args) => ArticleListSetScope(ArticleScope::from_str(args.as_str())?),
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "expected scope: all, unread or marked"
                    ));
                }
            },
            "scrape" => ArticleCurrentScrape,

            "quit" | "q" => ApplicationQuit,

            "/" => match args {
                Some(args) => ArticleListSearch(ArticleQuery::from_str(args.as_str())?),
                None => return Err(color_eyre::eyre::eyre!("search query expected")),
            },
            "/next" => ArticleListSearchNext,
            "/prev" => ArticleListSearchPrevious,

            "=" => match args {
                Some(args) => ArticleListFilterSet(ArticleQuery::from_str(args.as_str())?),
                None => return Err(color_eyre::eyre::eyre!("search query expected")),
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
