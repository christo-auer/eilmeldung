use std::collections::HashMap;

use news_flash::models::{Article, ArticleFilter, FatArticle, FeedID, Thumbnail};

use crate::{
    app::AppState,
    ui::{articles_list::ArticleScope, tooltip::Tooltip},
};

#[derive(Debug)]
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
    PanelFocusPrevious,
    PanelFocusNextCyclic,
    PanelFocusPreivousCyclic,
    ToggleDistractionFreeMode,

    // feeds and articles
    FeedsSync,
    ArticleOpenInBrowser,
    ArticleSetCurrentAsRead,
    ArticleSetCurrentAsUnread,
    ArticleCurrentToggleRead,
    ArticleListSelectNextUnread,
    ArticleListSetAllRead,
    ArticleListSetAllUnread,
    ArticleListSetScope(ArticleScope),
    ArticleScrape,

    // application
    ApplicationQuit,
}

#[derive(Debug)]
pub enum Event {
    ArticlesSelected(ArticleFilter),
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

    // messaging/status
    Tooltip(Tooltip),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),
}

#[derive(Debug)]
pub enum Message {
    Command(Command),
    Event(Event),
}

pub trait MessageReceiver {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()>;
}
