use std::collections::HashMap;

use news_flash::models::{Article, ArticleFilter, FatArticle, FeedID, Marked, Read, Thumbnail};

use crate::{
    app::AppState,
    ui::{articles_list::ArticleScope, tooltip::Tooltip},
};

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
    FocusNext,
    FocusPrevious,

    // feeds and articles
    Sync,
    ArticlesSelected(ArticleFilter),
    ArticleSelected(Article),
    FatArticleSelected(Article),
    OpenInBrowser,
    SetCurrentAsRead,
    SetCurrentAsUnread,
    ToggleCurrentRead,
    SelectNextUnread,
    SetAllRead,
    SetAllUnread,
    SetArticleScope(ArticleScope),

    AsyncSyncStarted,
    AsyncSyncFinished(HashMap<FeedID, i64>),

    AsyncFetchThumbnailStarted,
    AsyncFetchThumbnailFinished(Option<Thumbnail>),

    AsyncFetchFatArticleStarted,
    AsyncFetchFatArticleFinished(FatArticle),

    AsyncMarkArticlesAsReadStarted,
    AsyncMarkArticlesAsReadFinished,

    AsyncOperationFailed(String),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),
    ApplicationQuit,

    // messaging/status
    Tooltip(Tooltip),
}

pub trait CommandReceiver {
    async fn process_command(&mut self, command: &Command) -> color_eyre::Result<()>;
}
