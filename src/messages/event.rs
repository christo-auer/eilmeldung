use std::collections::HashMap;

use news_flash::{
    error::NewsFlashError,
    models::{Article, FatArticle, Feed, FeedID, Tag, Thumbnail},
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

    AsyncMarkArticlesAsRead,
    AsyncMarkArticlesAsReadFinished,

    AsyncMarkArticlesAsMarked,
    AsyncMarkArticlesAsMarkedFinished,

    AsyncOperationFailed(AsyncOperationError, Box<Event>),

    AsyncSetOffline,
    AsyncSetOfflineFinished(bool),

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),

    // connectivity
    ConnectionAvailable,
    ConnectionLost(ConnectionLostReason),
}
