use std::collections::HashMap;

use news_flash::{
    error::NewsFlashError,
    models::{Article, Category, FatArticle, Feed, FeedID, Tag, Thumbnail},
};
use ratatui::{crossterm::event::KeyEvent, text::Line, widgets::Paragraph};

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

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // help popup
    ShowHelpPopup(String, Vec<Line<'static>>),
    HideHelpPopup,

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
                | AsyncFeedAddFinished(_)
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
        )
    }
}
