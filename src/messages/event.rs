use std::collections::HashMap;

use news_flash::models::{Article, FatArticle, Feed, FeedID, Tag, Thumbnail};
use ratatui::crossterm::event::KeyEvent;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Event {
    ArticlesSelected(AugmentedArticleFilter),
    ArticleSelected(Article, Option<Feed>, Option<Vec<Tag>>),
    FatArticleSelected(Article, Option<Feed>, Option<Vec<Tag>>),

    AsyncSyncStarted,
    AsyncSyncFinished(HashMap<FeedID, i64>),

    AsyncFetchThumbnailStarted,
    AsyncFetchThumbnailFinished(Option<Thumbnail>),

    AsyncFetchFatArticleStarted,
    AsyncFetchFatArticleFinished(FatArticle),

    AsyncMarkArticlesAsReadStarted,
    AsyncMarkArticlesAsReadFinished,

    AsyncOperationFailed(String),

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),
}
