mod message_recv;
mod view;

pub mod prelude {
    pub use super::ArticlesList;
}

use message_recv::ArticleListModelData;
use view::ArticleListViewData;

use crate::prelude::*;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

pub struct ArticlesList {
    config: Arc<Config>,

    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    view_data: ArticleListViewData,
    model_data: ArticleListModelData,
}

impl ArticlesList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            news_flash_utils: news_flash_utils.clone(),
            message_sender,

            view_data: ArticleListViewData::new(config.article_scope),

            model_data: ArticleListModelData::default(),
        }
    }
}
