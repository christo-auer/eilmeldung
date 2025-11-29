use std::fmt::Display;
use std::str::FromStr;

use news_flash::models::Url;
use ratatui::style::{Color, ParseColorError};
use serde::Deserialize;
use strum::EnumMessage;

use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default, strum::EnumString, strum::EnumMessage)]
#[strum(serialize_all="snake_case",
    parse_err_fn = CommandParseError::panel_expected,
    parse_err_ty = CommandParseError)]
pub enum Panel {
    #[default]
    #[strum(serialize = "feeds")]
    #[strum(
        message = "feed list",
        detailed_message = "panel with tree of feeds, categories, tags, etc."
    )]
    FeedList,
    #[strum(serialize = "articles")]
    #[strum(
        message = "article list",
        detailed_message = "panel with the list of articles"
    )]
    ArticleList,
    #[strum(serialize = "content")]
    #[strum(
        message = "article content",
        detailed_message = "content of the selected article"
    )]
    ArticleContent,
}
impl Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_detailed_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

impl CommandParseError {
    fn panel_expected(_: &str) -> CommandParseError {
        CommandParseError::PanelExpected
    }
}

#[derive(Clone, Debug, Default, strum::EnumIter, strum::EnumMessage)]
pub enum ActionScope {
    #[default]
    #[strum(message = "current", detailed_message = "currently selected article")]
    Current,
    #[strum(message = "all", detailed_message = "all articles")]
    All,
    #[strum(
        message = "query",
        detailed_message = "all articles defined by a query"
    )]
    Query(ArticleQuery),
}

impl FromStr for ActionScope {
    type Err = CommandParseError;

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
    fn from_option_string(s: Option<&str>) -> Result<ActionScope, CommandParseError> {
        match s {
            Some(s) => Ok(ActionScope::from_str(s)?),
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

#[derive(Debug, Clone, Default, strum::EnumIter, strum::EnumMessage)]
pub enum ActionSetReadTarget {
    #[default]
    #[strum(message = "current", detailed_message = "currently selected panel")]
    Current,
    #[strum(message = "feed list", detailed_message = "feed list")]
    FeedList,
    #[strum(message = "article list", detailed_message = "article list")]
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
        self.get_detailed_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

#[derive(Debug, Clone, Copy, Default, strum::EnumString, strum::EnumIter, strum::EnumMessage)]
#[strum(serialize_all = "snake_case")]
pub enum PastePosition {
    #[default]
    #[strum(
        message = "after",
        detailed_message = "position after the current element"
    )]
    After,
    #[strum(
        message = "before",
        detailed_message = "position before the current element"
    )]
    Before,
}

impl Display for PastePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_detailed_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

#[derive(Clone, Debug, Default, strum::EnumString, strum::EnumIter, strum::EnumMessage)]
pub enum Command {
    #[default]
    #[strum(serialize = "nop")]
    NoOperation,

    // general navigation
    #[strum(
        serialize = "up",
        message = "up",
        detailed_message = "nagivates up in the current context (all)"
    )]
    NavigateUp,

    #[strum(
        serialize = "down",
        message = "down",
        detailed_message = "nagivates down in the current context (all)"
    )]
    NavigateDown,

    #[strum(
        serialize = "pageup",
        message = "pageup",
        detailed_message = "navigates up by several items (all)"
    )]
    NavigatePageUp,
    #[strum(
        serialize = "pagedown",
        message = "pagedown",
        detailed_message = "navigates down by several items (all)"
    )]
    NavigatePageDown,

    #[strum(
        serialize = "gotofirst",
        message = "gotofirst",
        detailed_message = "navigate to first element (all)"
    )]
    NavigateFirst,
    #[strum(
        serialize = "gotolast",
        message = "gotolast",
        detailed_message = "navigate to last element (all)"
    )]
    NavigateLast,
    #[strum(
        serialize = "left",
        message = "left",
        detailed_message = "nagivates left in the current context (all)"
    )]
    NavigateLeft,
    #[strum(
        serialize = "right",
        message = "right",
        detailed_message = "nagivates right in the current context (all)"
    )]
    NavigateRight,

    // Panels
    #[strum(
        serialize = "next",
        message = "next",
        detailed_message = "focuses the next panel until article content (all)"
    )]
    PanelFocusNext,
    #[strum(
        serialize = "prev",
        message = "prev",
        detailed_message = "focuses the previous panel until feed list (all)"
    )]
    PanelFocusPrevious,
    #[strum(
        serialize = "nextc",
        message = "nextc",
        detailed_message = "focuses the next panel, cycling back to feed list (all)"
    )]
    PanelFocusNextCyclic,
    #[strum(
        serialize = "prevc",
        message = "prevc",
        detailed_message = "focuses the previous panel, cycling back to article content (all)"
    )]
    PanelFocusPreviousCyclic,

    #[strum(
        serialize = "focus",
        message = "focus <panel>",
        detailed_message = "focuses the given panel (all)"
    )]
    PanelFocus(Panel),

    #[strum(
        serialize = "zen",
        message = "zen",
        detailed_message = "toggle distraction-free mode (article content)"
    )]
    ToggleDistractionFreeMode,

    // feed list
    #[strum(
        serialize = "sync",
        message = "sync",
        detailed_message = "sync all feeds (feed list)"
    )]
    FeedListSync,
    #[strum(
        serialize = "categoryadd",
        message = "categoryadd <category name>",
        detailed_message = "add a new category with the given name (feed list)"
    )]
    FeedListCategoryAdd(String),

    #[strum(
        serialize = "feedadd",
        message = "feedadd <feed URL> [<name>]",
        detailed_message = "add a new feed with the given URL and optional name (feed list)"
    )]
    FeedListFeedAdd(Option<Url>, Option<String>),

    #[strum(
        serialize = "tagchangecolor",
        message = "tagchangecolor <color>",
        detailed_message = "change the color of the selected tag (feed list)"
    )]
    FeedListTagChangeColor(Color),

    #[strum(
        serialize = "rename",
        message = "rename <new name>",
        detailed_message = "rename the selected item (feed list)"
    )]
    FeedListRenameEntity(String),

    #[strum(
        serialize = "remove",
        message = "remove",
        detailed_message = "remove the selected childless item (feed list)"
    )]
    FeedListRemoveEntity,

    #[strum(
        serialize = "remove!",
        message = "remove!",
        detailed_message = "remove the selected item with children (feed list)"
    )]
    FeedListRemoveEntityWithChildren,

    #[strum(
        serialize = "feedchangeurl",
        message = "feedchangeurl",
        detailed_message = "change URL of the selected feed (feed list)"
    )]
    FeedListFeedChangeUrl(Option<Url>),

    #[strum(
        serialize = "yank",
        message = "yank",
        detailed_message = "yank the selected item (feed or category) for moving (feed list)"
    )]
    FeedListYankFeedOrCategory,

    #[strum(
        serialize = "paste",
        message = "paste <paste position>",
        detailed_message = "paste the yanked item before/after selected item (feed list)"
    )]
    FeedListPasteFeedOrCategory(PastePosition),

    #[strum(
        serialize = "toggle",
        message = "toggle",
        detailed_message = "toggle selected item open/closed (feed list)"
    )]
    FeedListToggleExpand,

    #[strum(
        serialize = "read",
        message = "read <target> <scope>",
        detailed_message = "set all articles matching the scope in the target to read (feed list, article list)"
    )]
    ActionSetRead(ActionSetReadTarget, ActionScope),

    #[strum(
        serialize = "unread",
        message = "unread <scope>",
        detailed_message = "set all articles matching the scope to unread (feed list, article list)"
    )]
    ActionSetUnread(ActionScope),

    #[strum(
        serialize = "mark",
        message = "mark <scope>",
        detailed_message = "marks all articles matching the scope (article list)"
    )]
    ActionSetMarked(ActionScope),

    #[strum(
        serialize = "unmark",
        message = "unmark <scope>",
        detailed_message = "unmarks all articles matching the scope (article list)"
    )]
    ActionSetUnmarked(ActionScope),

    #[strum(
        serialize = "open",
        message = "open",
        detailed_message = "opens all articles matching the scope in the webbrowser (article list)"
    )]
    ActionOpenInBrowser(ActionScope),

    #[strum(
        serialize = "tag",
        message = "tag <tag name> <scope>",
        detailed_message = "adds the tag to all articles matching the scope (article list)"
    )]
    ActionTagArticles(ActionScope, String),

    #[strum(
        serialize = "untag",
        message = "untag <tag name> <scope>",
        detailed_message = "removes the tag from all articles matching the scope (article list)"
    )]
    ActionUntagArticles(ActionScope, String),

    #[strum(
        serialize = "tagadd",
        message = "tagadd <tag name> [<color>]",
        detailed_message = "adds a new tag with the given name and optional color (feed list)"
    )]
    TagAdd(String, Option<Color>),

    // article list commands
    #[strum(
        serialize = "nextunread",
        message = "nextunread",
        detailed_message = "selected next unread item (article list)"
    )]
    ArticleListSelectNextUnread,

    #[strum(
        serialize = "show",
        message = "show <article scope>",
        detailed_message = "show only articles in the article scope (article list)"
    )]
    ArticleListSetScope(ArticleScope),

    #[strum(
        serialize = "scrape",
        message = "scrape",
        detailed_message = "scrape the current article (article list, article content)"
    )]
    ArticleCurrentScrape,

    // article list searching
    #[strum(
        serialize = "/",
        message = "/ <article query>",
        detailed_message = "search for articles matching the query (article list)"
    )]
    ArticleListSearch(ArticleQuery),

    #[strum(
        serialize = "/next",
        message = "/next",
        detailed_message = "search for the next article matching the query (article list)"
    )]
    ArticleListSearchNext,

    #[strum(
        serialize = "/prev",
        message = "/prev",
        detailed_message = "search for the previous article matching the query (article list)"
    )]
    ArticleListSearchPrevious,

    #[strum(
        serialize = "=",
        message = "= <article query>",
        detailed_message = "filter articles by query (article list)"
    )]
    ArticleListFilterSet(ArticleQuery),

    #[strum(
        serialize = "=apply",
        message = "=apply",
        detailed_message = "apply current filter (article list)"
    )]
    ArticleListFilterApply,

    #[strum(
        serialize = "=clear",
        message = "=clear",
        detailed_message = "clear current filter (article list)"
    )]
    ArticleListFilterClear,

    // application
    #[strum(
        serialize = "quit",
        message = "quit",
        detailed_message = "quit eilmeldung (all)"
    )]
    ApplicationQuit,

    // command line
    #[strum(
        serialize = ":",
        message = ": [<command line content>]",
        detailed_message = "open command line with optional content (all)"
    )]
    CommandLineOpen(Option<String>),

    #[strum(
        serialize = "?",
        serialize = "confirm",
        message = "confirm <command>",
        detailed_message = "ask user for confirmation to execute command and, if positive, execute command (all)"
    )]
    CommandConfirm(Box<Command>),

    // redraw command
    #[strum(
        serialize = "redraw",
        message = "redraw",
        detailed_message = "redraw screen (all)"
    )]
    Redraw,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandParseError {
    #[error("expecting command")]
    CommandExpected,

    #[error("expecting command name")]
    CommandNameExpected(#[from] strum::ParseError),

    #[error("expecting tag")]
    TagExpected,

    #[error("expecting article scope")]
    ArticleScopeExpected,

    #[error("expecting color")]
    ColorExpected(#[from] ParseColorError),

    #[error("expecting URL")]
    URLExpected(#[from] url::ParseError),

    #[error("expecting panel")]
    PanelExpected,

    #[error("expecting article search query")]
    ArticleQueryExpected(#[from] QueryParseError),

    #[error("expecting a word")]
    WordExpected(String),

    #[error("expecting something")]
    SomethingExpected(String),

    #[error("unexpected")]
    NothingExcepted(String),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        match self.clone() {
            NoOperation => write!(f, "no operation"),
            NavigateUp => write!(f, "up"),
            NavigateDown => write!(f, "down"),
            NavigatePageUp => write!(f, "page up"),
            NavigatePageDown => write!(f, "page down"),
            NavigateFirst => write!(f, "to first"),
            NavigateLast => write!(f, "to last"),
            NavigateLeft => write!(f, "left"),
            NavigateRight => write!(f, "right"),
            PanelFocusNext => write!(f, "focus next"),
            PanelFocus(panel) => write!(f, "focus {}", panel),
            PanelFocusPrevious => write!(f, "focus previous"),
            PanelFocusNextCyclic => write!(f, "focus next"),
            PanelFocusPreviousCyclic => write!(f, "focus next"),
            ToggleDistractionFreeMode => write!(f, "distraction free mode"),
            FeedListToggleExpand => write!(f, "toggle selected node"),
            FeedListCategoryAdd(name) => write!(f, "add category {name}"),
            FeedListFeedAdd(Some(url), Some(name)) => write!(f, "add feed {name} with url {url}"),
            FeedListFeedAdd(Some(url), None) => write!(f, "add feed with url {url}"),
            FeedListFeedAdd(None, _) => unreachable!(),
            FeedListRenameEntity(name) => write!(f, "rename selected to {name}"),
            FeedListRemoveEntity => write!(f, "remove selected"),
            FeedListRemoveEntityWithChildren => write!(f, "remove selected and its children"),
            FeedListFeedChangeUrl(Some(url)) => write!(f, "change url of selected feed to {url}"),
            FeedListFeedChangeUrl(None) => unreachable!(),
            FeedListYankFeedOrCategory => write!(f, "yank selected feed or category"),
            FeedListPasteFeedOrCategory(position) => {
                write!(
                    f,
                    "paste yanked feed or category {position} selected element"
                )
            }
            FeedListTagChangeColor(color) => {
                write!(f, "change color of tag to {}", color)
            }
            ArticleListSelectNextUnread => write!(f, "select next unread"),
            ArticleListSetScope(ArticleScope::Marked) => write!(f, "show marked"),
            ArticleListSetScope(ArticleScope::Unread) => write!(f, "show unread"),
            ArticleListSetScope(ArticleScope::All) => write!(f, "show all"),
            ArticleCurrentScrape => write!(f, "scrape content"),
            ApplicationQuit => write!(f, "quit application"),
            Redraw => write!(f, "redraw ui"),
            CommandLineOpen(input) => write!(f, ":{}", input.unwrap_or_default()),
            ArticleListSearch(query) => write!(f, "search article by query: {}", query.query_string()),
            ArticleListSearchNext => write!(f, "article search next"),
            ArticleListSearchPrevious => write!(f, "article search previous"),
            ArticleListFilterSet(query) => {
                write!(f, "filter article list by query: {}", query.query_string())
            }
            ArticleListFilterApply => write!(f, "apply current article filter"),
            ArticleListFilterClear => write!(f, "clear article filter"),

            FeedListSync => write!(f, "sync all"),
            ActionSetRead(target, action_scope) => {
                write!(f, "mark {action_scope} as read in {target}",)
            }
            ActionSetUnread(action_scope) => write!(f, "mark {} as unread", action_scope),
            ActionSetMarked(action_scope) => write!(f, "mark {}", action_scope),
            ActionSetUnmarked(action_scope) => write!(f, "unmark {}", action_scope),
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
            CommandConfirm(command) => write!(f, "{}?", command),
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

impl<const N: usize> From<[Command; N]> for CommandSequence {
    fn from(value: [Command; N]) -> Self {
        value.into_iter().collect::<Vec<Command>>().into()
    }
}

impl<const N: usize> From<[&str; N]> for CommandSequence {
    fn from(value: [&str; N]) -> Self {
        value
            .iter()
            .map(|s| Command::from_str(s).unwrap()) // <- if this fails it should fail hard
            .collect::<Vec<Command>>()
            .into()
    }
}

impl From<&str> for CommandSequence {
    fn from(value: &str) -> Self {
        Command::from_str(value).unwrap().into() // <- if this fails it should fail hard
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

    (first.trim().to_owned(), args)
}

fn expect_word(s: &mut Option<String>, to_expect: &str) -> Result<String, CommandParseError> {
    let Some(args) = s.as_mut() else {
        return Err(CommandParseError::WordExpected(to_expect.to_owned()));
    };

    let (word, tail) = split_off_first(args.as_str());

    *s = tail;

    Ok(word)
}

fn expect_something(s: Option<String>, to_expect: &str) -> Result<String, CommandParseError> {
    s.ok_or(CommandParseError::SomethingExpected(to_expect.to_owned()))
}

fn expect_from_str<T: FromStr>(
    s: &mut Option<String>,
    to_expect: &str,
) -> Result<T, CommandParseError>
where
    T::Err: Into<CommandParseError>,
{
    let word = expect_word(s, to_expect)?;
    T::from_str(word.as_str()).map_err(|e| e.into())
}

fn expect_nothing(s: Option<String>) -> Result<(), CommandParseError> {
    match s {
        Some(s) => Err(CommandParseError::NothingExcepted(s)),
        None => Ok(()),
    }
}

impl Command {
    pub fn parse(s: &str) -> Result<Self, CommandParseError> {
        use CommandParseError as E;

        let mut args = if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        };

        let command: Command = expect_from_str(&mut args, "expecting command")?;

        use Command as C;
        Ok(match command {
            C::CommandConfirm(_) => {
                let Some(args) = args else {
                    return Err(E::CommandExpected);
                };
                C::CommandConfirm(Box::new(Command::parse(&args)?))
            }

            C::PanelFocus(_) => {
                let panel: Panel =
                    expect_from_str(&mut args, "expecting panel: feeds, articles, content")?;
                C::PanelFocus(panel)
            }

            C::ActionSetRead(..) => {
                let old_args = args.clone();
                let word = expect_word(&mut args, "target or scope");

                let target = word
                    .map(|word| match ActionSetReadTarget::from_str(word.as_str()) {
                        Ok(target) => target,
                        Err(_) => {
                            args = old_args; // restore old version of args
                            ActionSetReadTarget::Current
                        }
                    })
                    .unwrap_or(ActionSetReadTarget::Current);

                C::ActionSetRead(target, ActionScope::from_option_string(args.as_deref())?)
            }

            C::ActionSetUnread(..) => {
                C::ActionSetUnread(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionOpenInBrowser(..) => {
                C::ActionOpenInBrowser(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionSetMarked(..) => {
                C::ActionSetMarked(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionSetUnmarked(..) => {
                C::ActionSetMarked(ActionScope::from_option_string(args.as_deref())?)
            }
            tag_command @ (C::ActionTagArticles(..) | C::ActionUntagArticles(..)) => {
                let tag = expect_word(&mut args, "tag name")?;

                match tag_command {
                    C::ActionTagArticles(..) => {
                        C::ActionTagArticles(ActionScope::from_option_string(args.as_deref())?, tag)
                    }
                    C::ActionUntagArticles(..) => C::ActionUntagArticles(
                        ActionScope::from_option_string(args.as_deref())?,
                        tag,
                    ),
                    _ => unreachable!(),
                }
            }

            C::FeedListFeedAdd(..) => {
                let url = Url::new(expect_from_str::<reqwest::Url>(&mut args, "feed URL")?);
                let name = args;
                C::FeedListFeedAdd(Some(url), name)
            }

            C::FeedListFeedChangeUrl(..) => {
                let url = Url::new(expect_from_str::<reqwest::Url>(&mut args, "feed URL")?);
                expect_nothing(args)?;
                C::FeedListFeedChangeUrl(Some(url))
            }

            C::FeedListPasteFeedOrCategory(..) => {
                let position = expect_from_str::<PastePosition>(&mut args, "paste position")?;
                expect_nothing(args)?;
                C::FeedListPasteFeedOrCategory(position)
            }

            C::FeedListCategoryAdd(..) => {
                C::FeedListCategoryAdd(expect_something(args, "expecting category name")?)
            }

            C::FeedListRenameEntity(..) => {
                C::FeedListRenameEntity(expect_something(args, "expecting new name")?)
            }

            C::FeedListTagChangeColor(..) => {
                let color: Color = expect_from_str(&mut args, "tag color")?;
                expect_nothing(args)?;
                C::FeedListTagChangeColor(color)
            }

            C::TagAdd(..) => {
                let tag_title = expect_word(&mut args, "tag name")?;
                let color: Option<Color> = match args {
                    None => None,
                    _ => Some(expect_from_str(&mut args, "tag color")?),
                };
                expect_nothing(args)?;
                C::TagAdd(tag_title, color)
            }

            C::ArticleListSetScope(..) => C::ArticleListSetScope(expect_from_str::<ArticleScope>(
                &mut args,
                "scope expected",
            )?),

            C::ArticleListSearch(..) => C::ArticleListSearch(ArticleQuery::from_str(
                expect_something(args, "query expected")?.as_str(),
            )?),

            C::ArticleListFilterSet(..) => C::ArticleListFilterSet(ArticleQuery::from_str(
                expect_something(args, "article query expected")?.as_str(),
            )?),

            C::CommandLineOpen(..) => C::CommandLineOpen(args),

            command_without_args => command_without_args,
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
