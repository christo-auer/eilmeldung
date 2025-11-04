use crate::prelude::*;

impl FeedList {
    pub(super) fn generate_articles_selected_command(&self) -> color_eyre::Result<()> {
        if let Some(selected) = self.get_selected() {
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
        let selected_before_item = self.tree_state.selected().last().cloned();

        match message {
            Message::Command(NavigateUp) if self.is_focused => {
                self.tree_state.key_up();
            }
            Message::Command(NavigateDown) if self.is_focused => {
                self.tree_state.key_down();
            }
            Message::Command(NavigateFirst) if self.is_focused => {
                self.tree_state.select_first();
            }
            Message::Command(NavigateLast) if self.is_focused => {
                self.tree_state.select_last();
            }
            Message::Command(NavigateLeft) if self.is_focused => {
                self.tree_state.key_left();
            }
            Message::Command(NavigateRight) if self.is_focused => {
                self.tree_state.key_right();
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.tree_state
                    .scroll_down(self.config.input_config.scroll_amount);
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.tree_state
                    .scroll_up(self.config.input_config.scroll_amount);
            }

            Message::Event(ApplicationStarted)
            | Message::Event(AsyncSyncFinished(_))
            | Message::Event(AsyncMarkArticlesAsReadFinished) => {
                self.build_tree().await?;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::FeedSelection;
            }

            _ => (),
        };

        // get selection after
        let selected_after_item = self.tree_state.selected().last();

        if selected_before_item.as_ref() != selected_after_item {
            self.update_tooltip(selected_after_item)?;
            self.generate_articles_selected_command()?;
        }

        Ok(())
    }
}
