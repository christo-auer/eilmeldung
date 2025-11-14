mod feed_list_item;
mod model;
mod view;

pub mod prelude {
    pub use super::FeedList;
}

use feed_list_item::FeedListItem;

use crate::{
    prelude::*,
    ui::{
        feeds_list::{model::FeedListModelData, view::FeedListViewData},
        tooltip,
    },
};
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

pub struct FeedList {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,

    view_data: FeedListViewData,
    model_data: FeedListModelData,

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
            message_sender,
            model_data: FeedListModelData::new(news_flash_utils.clone()),
            view_data: FeedListViewData::default(),
            is_focused: false,
        }
    }
    pub(super) fn update_tooltip(
        &self,
        now_selected: Option<&FeedListItem>,
    ) -> color_eyre::Result<()> {
        if let Some(item) = now_selected {
            tooltip(
                &self.message_sender,
                item.to_tooltip(&self.config).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    pub(super) fn selected(&self) -> Option<FeedListItem> {
        self.view_data.tree_state().selected().last().cloned()
    }

    fn generate_articles_selected_command(&self) -> color_eyre::Result<()> {
        if let Some(selected) = self.selected() {
            match selected.try_into() {
                Ok(article_filter) => {
                    self.message_sender
                        .send(Message::Event(Event::ArticlesSelected(article_filter)))?;
                }
                Err(err) => {
                    tooltip(
                        &self.message_sender,
                        err.to_string().as_str(),
                        TooltipFlavor::Warning,
                    )?;
                }
            }
        };

        Ok(())
    }
}

impl MessageReceiver for FeedList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        // get selection before
        let selected_before_item = self.selected().clone();
        let mut models_need_update = false;

        match message {
            Message::Command(NavigateUp) if self.is_focused => {
                self.view_data.tree_state_mut().key_up();
            }
            Message::Command(NavigateDown) if self.is_focused => {
                self.view_data.tree_state_mut().key_down();
            }
            Message::Command(NavigateFirst) if self.is_focused => {
                self.view_data.tree_state_mut().select_first();
            }
            Message::Command(NavigateLast) if self.is_focused => {
                self.view_data.tree_state_mut().select_last();
            }
            Message::Command(NavigateLeft) if self.is_focused => {
                self.view_data.tree_state_mut().key_left();
            }
            Message::Command(NavigateRight) if self.is_focused => {
                self.view_data.tree_state_mut().key_right();
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.view_data
                    .tree_state_mut()
                    .scroll_down(self.config.input_config.scroll_amount);
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.view_data
                    .tree_state_mut()
                    .scroll_up(self.config.input_config.scroll_amount);
            }

            Message::Event(ApplicationStarted)
            | Message::Event(AsyncSyncFinished(_))
            | Message::Event(AsyncMarkArticlesAsReadFinished)
            | Message::Event(AsyncMarkArticlesAsMarkedFinished)
            | Message::Event(AsyncTagArticleFinished) => {
                models_need_update = true;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::FeedSelection;
            }

            _ => (),
        };

        // get selection after
        let mut selected_after_item = self.selected();

        if selected_after_item.is_none() {
            self.view_data.tree_state_mut().select_first();
            selected_after_item = self.selected();
        }

        if models_need_update {
            let model_data = &mut self.model_data;
            model_data.update().await?;
            self.view_data
                .update(&self.config, &self.model_data)
                .await?;
        }

        if selected_before_item.as_ref() != selected_after_item.as_ref() {
            self.update_tooltip(selected_after_item.as_ref())?;
            self.generate_articles_selected_command()?;
        }

        Ok(())
    }
}
