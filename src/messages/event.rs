use std::collections::HashMap;

use news_flash::models::{Article, FatArticle, Feed, FeedID, Tag, Thumbnail};
use ratatui::crossterm::event::KeyEvent;

use crate::prelude::*;

#[derive(Clone, Debug)]
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

    AsyncOperationFailed(String, Box<Event>),

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // raw key event
    Key(KeyEvent),
}
