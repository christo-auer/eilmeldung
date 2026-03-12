use std::collections::HashMap;

use news_flash::{
    error::NewsFlashError,
    models::{ArticleID, Category, FatArticle, Feed, FeedID, Tag, Thumbnail},
};
use ratatui::{
    crossterm::event::{KeyEvent, MouseEvent},
    text::Text,
};

use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum AsyncOperationError {
    #[error("news flash error")]
    NewsFlashError(#[from] NewsFlashError),

    #[error("error report")]
    Report(#[from] color_eyre::Report),
}

#[derive(Debug)]
pub enum Event {
    ArticlesSelected(AugmentedArticleFilter),
    ArticleSelected(ArticleID),
    FatArticleSelected(ArticleID),

    AsyncSync,
    AsyncSyncFinished(HashMap<FeedID, i64>),

    AsyncArticleThumbnailFetch,
    AsyncArticleThumbnailFetchFinished(Option<Thumbnail>),

    AsyncArticleFatFetch,
    AsyncArticleFatFetchFinished(FatArticle),

    AsyncArticlesMark,
    AsyncArticlesMarkFinished,

    AsyncArticleTag,
    AsyncArticleTagFinished,

    AsyncArticleUntag,
    AsyncArticleUntagFinished,

    AsyncTagAdd,
    AsyncTagAddFinished(Tag),

    AsyncTagRemove,
    AsyncTagRemoveFinished,

    AsyncFeedAdd,
    AsyncFeedAddFinished(Feed),

    AsyncFeedFetch,
    AsyncFeedFetchFinished(FeedID, i64),

    AsyncCategoryAdd,
    AsyncCategoryAddFinished(Category),

    AsyncFeedRename,
    AsyncRenameFeedFinished(Feed),

    AsyncCategoryRename,
    AsyncCategoryRenameFinished(Category),

    AsyncCategoryRemove,
    AsyncCategoryRemoveFinished,

    AsyncFeedRemove,
    AsyncFeedRemoveFinished,

    AsyncFeedUrlChange,
    AsyncFeedUrlChangeFinished,

    AsyncTagEdit,
    AsyncTagEditFinished(Tag),

    AsyncOperationFailed(AsyncOperationError, Box<Event>),

    AsyncSetOffline,
    AsyncSetOfflineFinished(bool),

    AsyncSetAllRead,
    AsyncSetAllReadFinished,

    AsyncFeedSetRead,
    AsyncFeedSetReadFinished,

    AsyncCategorySetRead,
    AsyncCategorySetReadFinished,

    AsyncTagSetRead,
    AsyncTagSetReadFinished,

    AsyncArticlesSetRead,
    AsyncArticlesSetReadFinished,

    AsyncFeedMove,
    AsyncFeedMoveFinished,

    AsyncCategoryMove,
    AsyncCategoryMoveFinished,

    AsyncImportOpml,
    AsyncImportOpmlFinished,

    AsyncLogout,
    AsyncLogoutFinished,

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // help popup
    ShowHelpPopup(String, Text<'static>),
    ShowModalHelpPopup(String, Text<'static>),
    HideHelpPopup,

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),

    // raw mouse event
    Mouse(MouseEvent),

    // mouse click on article list at row offset from top of inner area
    MouseArticleClick(u16),

    // mouse click on feed list at screen position (col, row)
    MouseFeedClick(u16, u16),

    // mouse scroll viewport without moving selection (panel, lines)
    MouseScrollUp(Panel),
    MouseScrollDown(Panel),

    // terminal resized
    Resized(u16, u16),

    // connectivity
    ConnectionAvailable,
    ConnectionLost(ConnectionLostReason),
}

impl Event {
    pub fn caused_model_update(&self) -> bool {
        use Event::*;

        matches!(
            self,
            AsyncSyncFinished(_)
                | AsyncFeedAddFinished(_)
                | AsyncFeedFetchFinished(..)
                | AsyncRenameFeedFinished(_)
                | AsyncCategoryRenameFinished(_)
                | AsyncArticleFatFetchFinished(_)
                | AsyncArticlesMarkFinished
                | AsyncArticleTagFinished
                | AsyncArticleUntagFinished
                | AsyncTagAddFinished(_)
                | AsyncTagRemoveFinished
                | AsyncTagEditFinished(_)
                | AsyncOperationFailed(..)
                | AsyncSetOfflineFinished(_)
                | AsyncSetAllReadFinished
                | AsyncFeedSetReadFinished
                | AsyncCategorySetReadFinished
                | AsyncTagSetReadFinished
                | AsyncArticlesSetReadFinished
                | AsyncImportOpmlFinished,
        )
    }
}
