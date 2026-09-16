use std::{collections::HashMap, process::ExitStatus, sync::Arc};

use news_flash::{
    DiscoverResult,
    error::NewsFlashError,
    models::{ArticleID, Category, FatArticle, Feed, FeedID, Tag, Thumbnail},
};
use ratatui_image::picker::Picker;

use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum AsyncOperationError {
    #[error("news flash error: #1")]
    NewsFlashError(#[from] NewsFlashError),

    #[error("error report")]
    Report(#[from] color_eyre::Report),

    #[error("feed parse error: #1")]
    FeedParseError(#[from] news_flash::error::FeedParserError),
}

#[derive(Debug)]
pub enum Event {
    ArticlesSelected(AugmentedArticleFilter),
    ArticleSelected(Option<ArticleID>),

    AsyncSync,
    AsyncSyncFinished(HashMap<FeedID, Vec<ArticleID>>),

    AsyncArticleThumbnailFetch,
    AsyncArticleThumbnailFetchFinished(Option<Thumbnail>),

    AsyncArticleFatFetch,
    AsyncArticleFatFetchFinished(FatArticle),

    AsyncPipeArticle,
    AsyncPipeArticleFinished(ArticleID, ExitStatus, Option<String>, Option<String>),

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

    AsyncDiscoverFeeds,
    AsyncDiscoverFeedsFinished(DiscoverResult),

    FeedSelected(Feed),

    AsyncFeedFetch,
    AsyncFeedFetchFinished(FeedID, Vec<ArticleID>),

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

    ConfigFileChanged,
    ConfigReloaded(Arc<Config>),

    Tick, // general tick for animations and regular updates

    // messaging/status
    Tooltip(Tooltip<'static>),

    // popup related events
    Popup(PopupEvent),

    // application
    ApplicationStarted,
    ApplicationStateChanged(AppState),

    // mouse click on article list at row offset from top of inner area
    MouseArticleClick(u16),

    // mouse click on feed list at screen position (col, row)
    MouseFeedClick(u16, u16),

    // mouse scroll viewport without moving selection (panel, lines)
    MouseScrollUp(Panel),
    MouseScrollDown(Panel),

    // terminal resized
    Resized(u16, u16),

    // new image protocol picker
    ImageProtocolPickerUpdated(Picker),

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
                | AsyncPipeArticleFinished(..)
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
