mod parse;

use ratatui::style::Color;
use std::fmt::Display;
use std::str::FromStr;
use strum::EnumMessage;

use news_flash::models::Url;

pub mod prelude {
    pub use super::parse::CommandParseError;
    pub use super::{ActionScope, ActionTarget, Command, CommandSequence, Panel, PastePosition};
}

use crate::prelude::*;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    strum::EnumString,
    strum::EnumMessage,
    strum::EnumIter,
    strum::AsRefStr,
)]
#[strum(
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
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

impl CommandParseError {
    fn panel_expected(_: &str) -> CommandParseError {
        CommandParseError::PanelExpected
    }
}

#[derive(Clone, Debug, Default, strum::EnumIter, strum::EnumMessage, strum::AsRefStr)]
pub enum ActionScope {
    #[default]
    #[strum(
        serialize = "current",
        message = "current",
        detailed_message = "currently selected item"
    )]
    Current,
    #[strum(serialize = "all", message = "all", detailed_message = "all items")]
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

#[derive(Debug, Clone, Default, strum::EnumIter, strum::EnumMessage, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ActionTarget {
    #[default]
    #[strum(message = ".", detailed_message = "currently selected panel")]
    Current,
    #[strum(message = "feeds", detailed_message = "feed list")]
    FeedList,
    #[strum(message = "articles", detailed_message = "article list")]
    ArticleList,
}

impl FromStr for ActionTarget {
    type Err = color_eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ActionTarget::*;

        Ok(match s {
            "." => Current,
            "feeds" => FeedList,
            "articles" => ArticleList,
            _ => return Err(color_eyre::eyre::eyre!("unknown target {s}")),
        })
    }
}

impl Display for ActionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

#[derive(Debug, Clone, Copy, Default, strum::EnumString, strum::EnumIter, strum::EnumMessage)]
#[strum(serialize_all = "lowercase")]
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
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

#[derive(
    Clone, Debug, Default, strum::AsRefStr, strum::EnumString, strum::EnumIter, strum::EnumMessage,
)]
pub enum Command {
    #[default]
    #[strum(
        serialize = "nop",
        message = "nop",
        detailed_message = "no operation (for unmapping key bindings)"
    )]
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
        detailed_message = "nagivate left in the current context (all)"
    )]
    NavigateLeft,

    #[strum(
        serialize = "right",
        message = "right",
        detailed_message = "nagivate right in the current context (all)"
    )]
    NavigateRight,

    #[strum(
        serialize = "_search",
        message = "_search",
        detailed_message = "open prompt to search"
    )]
    InputSearch,

    #[strum(
        serialize = "_abort",
        message = "abort",
        detailed_message = "abort current dialog"
    )]
    InputAbort,

    #[strum(
        serialize = "_submit",
        message = "submit",
        detailed_message = "submit current input"
    )]
    InputSubmit,

    #[strum(
        serialize = "_clear",
        message = "clear",
        detailed_message = "clear current input"
    )]
    InputClear,

    // Panels
    #[strum(
        serialize = "next",
        message = "next",
        detailed_message = "focus next panel until article content (all)"
    )]
    PanelFocusNext,
    #[strum(
        serialize = "prev",
        message = "prev",
        detailed_message = "focus previous panel until feed list (all)"
    )]
    PanelFocusPrevious,
    #[strum(
        serialize = "nextc",
        message = "nextc",
        detailed_message = "focus next panel, cycling back to feed list (all)"
    )]
    PanelFocusNextCyclic,
    #[strum(
        serialize = "prevc",
        message = "prevc",
        detailed_message = "focus previous panel, cycling back to article content (all)"
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
        serialize = "removeall",
        message = "removeall",
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
        serialize = "expandcategories",
        message = "expandcategories <scope>",
        detailed_message = "expands current selected item or all categories belonging to scope (feed list)"
    )]
    FeedListExpandCategories(ArticleScope),

    #[strum(
        serialize = "expand",
        message = "expand",
        detailed_message = "expands current selected item (feed list)"
    )]
    FeedListExpand,

    #[strum(
        serialize = "collapse",
        message = "collapse",
        detailed_message = "collapses the currently selected item (feed list)"
    )]
    FeedListCollapse,

    #[strum(
        serialize = "collapseall",
        message = "collapseall",
        detailed_message = "collapses all items (feed list)"
    )]
    FeedListCollapseAll,

    #[strum(
        serialize = "read",
        message = "read [[<target>] <scope>]",
        detailed_message = "set all articles matching the scope in the target to read (feed list, article list)"
    )]
    ActionSetRead(ActionTarget, ActionScope),

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
        message = "tag <tag name> [<scope>]",
        detailed_message = "adds the tag to all articles matching the scope (article list)"
    )]
    ActionTagArticles(ActionScope, String),

    #[strum(
        serialize = "untag",
        message = "untag <tag name> [<scope>]",
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
        message = "show <target> <article scope>",
        detailed_message = "show only articles in the article scope (feed list, article list)"
    )]
    Show(ActionTarget, ArticleScope),

    #[strum(
        serialize = "scrape",
        message = "scrape",
        detailed_message = "scrape the current article (article list, article content)"
    )]
    ArticleCurrentScrape,

    // article list searching
    #[strum(
        serialize = "searcharticles",
        message = "searcharticles <article query>",
        detailed_message = "search for articles matching the query (article list)"
    )]
    ArticleListSearch(ArticleQuery),

    #[strum(
        serialize = "search",
        message = "search [<search term>]",
        detailed_message = "search current panel by search term; empty seach term to clear search (all)"
    )]
    Search(Option<SearchTerm>),

    #[strum(
        serialize = "searchnext",
        message = "searchnext",
        detailed_message = "search next matching item (all)"
    )]
    SearchNext,

    #[strum(
        serialize = "searchprev",
        message = "searchprev",
        detailed_message = "search previous matching item (all)"
    )]
    SearchPrevious,

    #[strum(
        serialize = "filter",
        message = "filter <article query>",
        detailed_message = "filter articles by query (article list)"
    )]
    ArticleListFilterSet(ArticleQuery),

    #[strum(
        serialize = "filterapply",
        message = "filterapply",
        detailed_message = "apply current filter (article list)"
    )]
    ArticleListFilterApply,

    #[strum(
        serialize = "filterclear",
        message = "filterclear",
        detailed_message = "clear current filter (article list)"
    )]
    ArticleListFilterClear,

    #[strum(
        serialize = "sort",
        message = "sort <sort order>",
        detailed_message = "sort articles according to sort order (article list)"
    )]
    ArticleListSort(SortOrder),

    #[strum(
        serialize = "sortreverse",
        message = "sortreverse",
        detailed_message = "reverse the current sort order (article list)"
    )]
    ArticleListSortReverse,

    #[strum(
        serialize = "sortclear",
        message = "sortclear",
        detailed_message = "clear the current sort order (article list)"
    )]
    ArticleListSortClear,

    #[strum(
        serialize = "query",
        message = "query <article query>",
        detailed_message = "executes a query on all articles (article list)"
    )]
    ArticleListQuery(ArticleQuery),

    #[strum(
        serialize = "share",
        message = "share <target>",
        detailed_message = "shares title and url with target (article list, article content)"
    )]
    ArticleShare(String),

    #[strum(
        serialize = "importopml",
        message = "importopml <path>",
        detailed_message = "imports an OPML file from the given path (all)"
    )]
    ImportOpml(String),

    #[strum(
        serialize = "exportopml",
        message = "exportopml <path>",
        detailed_message = "exports an OPML file to the given path (all)"
    )]
    ExportOpml(String),

    // application
    #[strum(
        serialize = "quit",
        message = "quit",
        detailed_message = "quit eilmeldung (all)"
    )]
    ApplicationQuit,

    // command line
    #[strum(
        serialize = "cmd",
        message = "cmd [<command line content>]",
        detailed_message = "open command line with optional content (all)"
    )]
    #[allow(clippy::enum_variant_names)]
    CommandLineOpen(Option<String>),

    #[strum(
        serialize = "LOGOUT",
        message = "LOGOUT <confirmation>",
        detailed_message = "logout, NOTE: this will remove ALL LOCAL DATA! Pass `NOW` as parameter to confirm."
    )]
    Logout(String),

    #[strum(
        serialize = "helpinput",
        message = "helpinput",
        detailed_message = "show help on in put mappings (all)"
    )]
    HelpInput,

    #[strum(
        serialize = "confirm",
        message = "confirm <command>",
        detailed_message = "ask user for confirmation to execute command and, if positive, execute command (all)"
    )]
    #[allow(clippy::enum_variant_names)]
    CommandConfirm(Box<Command>),

    #[strum(
        serialize = "in",
        message = "in <panel> <command>",
        detailed_message = "execute command in the panel"
    )]
    In(Panel, Box<Command>),

    // redraw command
    #[strum(
        serialize = "redraw",
        message = "redraw",
        detailed_message = "redraw screen (all)"
    )]
    Redraw,

    #[strum(
        serialize = "refresh",
        message = "refresh",
        detailed_message = "refreshes contents of all panels according to selections (all)"
    )]
    Refresh,
}

impl Command {
    pub fn unwrap_in(&self, panel: Panel) -> Option<&Command> {
        if let Command::In(target_panel, command) = self {
            return (*target_panel == panel).then_some(command);
        }
        Some(self)
    }
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
            InputSearch => write!(f, "open find prompt"),
            InputAbort => write!(f, "abort current input"),
            InputSubmit => write!(f, "submit current input"),
            InputClear => write!(f, "clear current input"),
            PanelFocusNext => write!(f, "focus next"),
            PanelFocus(panel) => write!(f, "focus {}", panel),
            PanelFocusPrevious => write!(f, "focus previous"),
            PanelFocusNextCyclic => write!(f, "focus previous (wrapping)"),
            PanelFocusPreviousCyclic => write!(f, "focus next (wrapping)"),
            ToggleDistractionFreeMode => write!(f, "distraction free mode"),
            FeedListToggleExpand => write!(f, "toggle selected node"),
            FeedListExpand => write!(f, "expand the selected node"),
            FeedListCollapse => write!(f, "collapse the selected node"),
            FeedListCollapseAll => write!(f, "collapse the selected node"),
            FeedListExpandCategories(ArticleScope::All) => write!(f, "expand all nodes"),
            FeedListExpandCategories(ArticleScope::Unread) => {
                write!(f, "expand all categories with unread items")
            }
            FeedListExpandCategories(ArticleScope::Marked) => {
                write!(f, "expand all categories with marked items")
            }
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
            Show(target, ArticleScope::Marked) => write!(f, "show only marked in {target}"),
            Show(target, ArticleScope::Unread) => write!(f, "show only unread in {target}"),
            Show(target, ArticleScope::All) => write!(f, "show all in {target}"),
            ArticleShare(share_target) => write!(f, "share article to {share_target}"),

            ArticleCurrentScrape => write!(f, "scrape content"),

            ImportOpml(path) => write!(f, "import OPML file from {path}"),
            ExportOpml(path) => write!(f, "export OPML file to {path}"),

            ApplicationQuit => write!(f, "quit application"),
            Redraw => write!(f, "redraw UI"),
            Refresh => write!(f, "refresh UI"),
            CommandLineOpen(input) => write!(f, ":{}", input.unwrap_or_default()),
            ArticleListSearch(query) => {
                write!(f, "search article by query: {}", query.query_string())
            }
            SearchNext => write!(f, "article search next"),
            SearchPrevious => write!(f, "article search previous"),
            Search(Some(search_term)) => write!(f, "search for {search_term}"),
            Search(None) => write!(f, "clear search"),
            ArticleListFilterSet(query) => {
                write!(f, "filter article list by query: {}", query.query_string())
            }
            ArticleListQuery(query) => {
                write!(f, "query all articles by: {}", query.query_string())
            }
            ArticleListFilterApply => write!(f, "apply current article filter"),
            ArticleListFilterClear => write!(f, "clear article filter"),
            ArticleListSort(sort_order) => write!(f, "sort article list by {}", sort_order),
            ArticleListSortReverse => write!(f, "reverse current sort order"),
            ArticleListSortClear => write!(f, "clear current sort order"),

            FeedListSync => write!(f, "sync all"),
            ActionSetRead(target, action_scope) => {
                write!(
                    f,
                    "mark {action_scope} as read in {}",
                    target
                        .get_detailed_message()
                        .unwrap_or(target.to_string().as_str())
                )
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
            Logout(_) => write!(
                f,
                "logout from provider, NOTE: this will remove ALL LOCAL DATA!"
            ),
            HelpInput => write!(f, "show help on input mappings"),
            TagAdd(tag_title, _) => {
                write!(f, "add tag #{}", tag_title)
            }
            CommandConfirm(command) => write!(f, "{}?", command),
            In(panel, command) => write!(f, "{command} in {panel}"),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Default)]
#[serde(transparent)]
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
