use std::collections::HashMap;

use news_flash::{
    error::NewsFlashError,
    models::{Article, Category, FatArticle, Feed, FeedID, Tag, Thumbnail},
};
use ratatui::crossterm::event::KeyEvent;

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
    ArticleSelected(Article, Option<Feed>, Option<Vec<Tag>>),
    FatArticleSelected(Article, Option<Feed>, Option<Vec<Tag>>),

    AsyncSync,
    AsyncSyncFinished(HashMap<FeedID, i64>),

    AsyncFetchThumbnail,
    AsyncFetchThumbnailFinished(Option<Thumbnail>),

    AsyncFetchFatArticle,
    AsyncFetchFatArticleFinished(FatArticle),

    AsyncMarkArticlesAsMarked,
    AsyncMarkArticlesAsMarkedFinished,

    AsyncTagArticle,
    AsyncTagArticleFinished,

    AsyncUntagArticle,
    AsyncUntagArticleFinished,

    AsyncTagAdd,
    AsyncTagAddFinished(Tag),

    AsyncTagRemove,
    AsyncTagRemoveFinished,

    AsyncAddFeed,
    AsyncAddFeedFinished(Feed),

    AsyncAddCategory,
    AsyncAddCategoryFinished(Category),

    AsyncRenameFeed,
    AsyncRenameFeedFinished(Feed),

    AsyncRenameCategory,
    AsyncRenameCategoryFinished(Category),

    AsyncTagEdit,
    AsyncTagEditFinished(Tag),

    AsyncOperationFailed(AsyncOperationError, Box<Event>),

    AsyncSetOffline,
    AsyncSetOfflineFinished(bool),

    AsyncSetAllRead,
    AsyncSetAllReadFinished,

    AsyncSetFeedRead,
    AsyncSetFeedReadFinished,

    AsyncSetCategoryRead,
    AsyncSetCategoryReadFinished,

    AsyncSetTagRead,
    AsyncSetTagReadFinished,

    AsyncSetArticlesAsRead,
    AsyncSetArticlesAsReadFinished,

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),

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
                | AsyncAddFeedFinished(_)
                | AsyncRenameFeedFinished(_)
                | AsyncRenameCategoryFinished(_)
                | AsyncFetchFatArticleFinished(_)
                | AsyncMarkArticlesAsMarkedFinished
                | AsyncTagArticleFinished
                | AsyncUntagArticleFinished
                | AsyncTagAddFinished(_)
                | AsyncTagRemoveFinished
                | AsyncTagEditFinished(_)
                | AsyncOperationFailed(..)
                | AsyncSetOfflineFinished(_)
                | AsyncSetAllReadFinished
                | AsyncSetFeedReadFinished
                | AsyncSetCategoryReadFinished
                | AsyncSetTagReadFinished
                | AsyncSetArticlesAsReadFinished
        )
    }
}
