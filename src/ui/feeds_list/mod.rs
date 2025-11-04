mod feed_list_item;
mod message_recv;
mod view;

pub mod prelude {
    pub use super::FeedList;
}

use feed_list_item::FeedListItem;

use crate::prelude::*;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::{TreeItem, TreeState};

pub struct FeedList {
    config: Arc<Config>,
    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    tree_state: TreeState<FeedListItem>,
    items: Vec<TreeItem<'static, FeedListItem>>,

    is_focused: bool,
}

impl FeedList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config,
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            items: vec![],
            tree_state: TreeState::default(),
            is_focused: true,
        }
    }
}
