mod model;
mod view;

pub mod prelude {
    pub use super::ArticlesList;
}

use crate::ui::articles_list::{model::ArticleListModelData, view::FilterState};
use news_flash::models::{Article, ArticleID, Read};
use view::ArticleListViewData;

use crate::prelude::*;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

pub struct ArticlesList {
    config: Arc<Config>,

    message_sender: UnboundedSender<Message>,

    view_data: ArticleListViewData<'static>,
    filter_state: FilterState,
    model_data: ArticleListModelData,

    is_focused: bool,
}

impl ArticlesList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            message_sender,

            view_data: ArticleListViewData::default(),
            filter_state: FilterState::new(config.article_scope),

            model_data: ArticleListModelData::new(news_flash_utils.clone()),

            is_focused: false,
        }
    }

    pub(super) fn restore_sensible_selection(
        &mut self,
        article_id: Option<&ArticleID>,
    ) -> color_eyre::Result<()> {
        // save offset distance
        let offset = *self.view_data.get_table_state_mut().offset_mut();
        let offset_distance = self
            .view_data
            .get_table_state_mut()
            .selected()
            .unwrap_or(0)
            .saturating_sub(offset);

        // first, we try to select the article with article_id
        if let Some(article_id) = article_id
            && let Some(index) = self
                .model_data
                .articles()
                .iter()
                .position(|article| article.article_id == *article_id)
        {
            *self.view_data.get_table_state_mut().offset_mut() =
                index.saturating_sub(offset_distance);
            return self.select_index_and_send_message(Some(index));
        }

        // the previous article is not there, next we select the first unread article
        self.view_data.get_table_state_mut().select(Some(0));
        self.select_next_unread()?;

        Ok(())
    }

    pub(super) fn select_index_and_send_message(
        &mut self,
        index: Option<usize>,
    ) -> color_eyre::Result<()> {
        let index = index
            .or(self.view_data.get_table_state().selected())
            .unwrap_or_default();
        if let Some(article) = self.model_data.articles().get(index) {
            self.view_data.table_state_mut().select(Some(index));
            self.message_sender
                .send(Message::Event(Event::ArticleSelected(
                    article.clone(),
                    self.model_data.feed_map().get(&article.feed_id).cloned(),
                    self.model_data
                        .tags_for_article()
                        .get(&article.article_id)
                        .map(|tag_ids| {
                            tag_ids
                                .iter()
                                .filter_map(|tag_id| self.model_data.tag_map().get(tag_id))
                                .cloned()
                                .collect()
                        })
                        .clone(),
                )))?;
        }

        self.adjust_offset();
        Ok(())
    }

    pub(super) fn select_next_unread(&mut self) -> color_eyre::Result<()> {
        let select = self.first_unread();
        self.select_index_and_send_message(select)
    }

    fn first_unread(&self) -> Option<usize> {
        let current_index = self.view_data.table_state().selected().unwrap_or(0);

        self.model_data
            .articles()
            .iter()
            .enumerate()
            .find(|(index, article)| *index >= current_index && article.unread == Read::Unread)
            .map(|(index, _)| index)
    }

    pub(super) fn open_in_browser(&self) -> color_eyre::Result<()> {
        if let Some(article) = self.get_current_article()
            && let Some(url) = article.url.as_ref()
        {
            webbrowser::open(url.as_ref())?;
        }

        // TODO error handling
        Ok(())
    }

    pub(super) fn get_current_article(&self) -> Option<Article> {
        if let Some(index) = self.view_data.get_table_state().selected() {
            return self.model_data.articles().get(index).cloned();
        }

        None
    }

    pub(super) fn search(
        &self,
        articles: &[Article],
        article_query: &ArticleQuery,
        reversed: bool,
    ) -> Option<usize> {
        let predicate = |article: &Article| {
            article_query.test(
                article,
                self.model_data.feed_map(),
                self.model_data.tags_for_article(),
                self.model_data.tag_map(),
            )
        };

        if !reversed {
            articles.iter().position(predicate)
        } else {
            articles.iter().rposition(predicate)
        }
    }

    fn adjust_offset(&mut self) {
        let Some(index) = self.view_data.get_table_state_mut().selected() else {
            return;
        };
        let offset = self.view_data.get_table_state_mut().offset_mut();
        let max_lines_above = self.config.theme.articles_list_height_lines as usize
            - (self.config.articles_list_visible_articles_after_selection + 1);

        if index.saturating_sub(*offset) > max_lines_above {
            *offset = index.saturating_sub(max_lines_above);
        }
    }

    pub(super) async fn set_current_read_status(
        &mut self,
        read: Option<Read>,
    ) -> color_eyre::Result<()> {
        if let Some(index) = self.view_data.get_table_state_mut().selected() {
            return self.model_data.set_read_status(index, read);
        }

        Ok(())
    }

    pub(super) fn search_next(
        &mut self,
        skip_current: bool,
        reversed: bool,
    ) -> color_eyre::Result<()> {
        let offset = if skip_current { 1 } else { 0 };
        let Some(article_query) = self.filter_state.article_search_query().as_ref() else {
            return Err(color_eyre::eyre::eyre!("no search query"));
        };

        if let Some(selected) = self.view_data.get_table_state_mut().selected() {
            let split_index = if !reversed {
                selected + offset
            } else {
                selected.saturating_sub(offset)
            };

            let slices = self.model_data.articles().split_at(split_index);

            let (first_range, second_range) = if reversed {
                slices
            } else {
                (slices.1, slices.0)
            };

            let (first_offset, second_offset) = if !reversed {
                (split_index, 0)
            } else {
                (0, split_index)
            };

            match self.search(first_range, article_query, reversed) {
                Some(index) => {
                    return self.select_index_and_send_message(Some(index + first_offset));
                }
                None => match self.search(second_range, article_query, reversed) {
                    Some(index) => {
                        tooltip(
                            &self.message_sender,
                            if !reversed {
                                "end reached, starting from beginning"
                            } else {
                                "beginning reached, starting from end"
                            },
                            TooltipFlavor::Warning,
                        )?;
                        return self.select_index_and_send_message(Some(index + second_offset));
                    }
                    None => {
                        tooltip(
                            &self.message_sender,
                            "no match found",
                            TooltipFlavor::Warning,
                        )?;
                    }
                },
            }
        }
        {}

        Ok(())
    }
}

impl crate::messages::MessageReceiver for ArticlesList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        let current_article = self.get_current_article().map(|article| article.article_id);
        let mut model_needs_update = false;
        let mut view_needs_update = false;

        // TODO refactor state mgmt
        match message {
            Message::Command(NavigateUp) if self.is_focused => {
                self.view_data.get_table_state_mut().select_previous();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateDown) if self.is_focused => {
                self.view_data.get_table_state_mut().select_next();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.view_data
                    .get_table_state_mut()
                    .scroll_up_by(self.config.theme.articles_list_height_lines - 1);
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.view_data.get_table_state_mut().scroll_down_by(
                    self.config.theme.articles_list_height_lines
                        - self.config.articles_list_visible_articles_after_selection as u16,
                );
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateFirst) if self.is_focused => {
                self.view_data.get_table_state_mut().select_first();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateLast) if self.is_focused => {
                self.view_data.get_table_state_mut().select_last();
                // manually "select" as select_last does not know the number of rows
                self.select_index_and_send_message(Some(self.model_data.articles().len() - 1))?;
            }

            Message::Event(AsyncOperationFailed(_, _)) => {
                model_needs_update = true;
            }

            Message::Event(ArticlesSelected(augmented_article_filter)) => {
                self.filter_state
                    .on_new_article_filter(augmented_article_filter.clone());
                model_needs_update = true;
            }

            Message::Command(ArticleListSetScope(scope)) => {
                *self.filter_state.article_scope_mut() = *scope;
                model_needs_update = true;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::ArticleSelection;
                view_needs_update = true;
            }

            Message::Command(ArticleCurrentOpenInBrowser) => {
                self.open_in_browser()?;
            }

            Message::Command(ArticleCurrentSetRead) => {
                self.set_current_read_status(Some(Read::Read)).await?;
                view_needs_update = true;
            }

            Message::Command(ArticleCurrentSetUnread) => {
                self.set_current_read_status(Some(Read::Unread)).await?;
                view_needs_update = true;
            }

            Message::Command(ArticleCurrentToggleRead) => {
                self.set_current_read_status(None).await?;
                view_needs_update = true;
            }

            Message::Command(ArticleListSetAllRead) => {
                self.model_data.set_all_read_status(Read::Read)?;
                view_needs_update = true;
            }

            Message::Command(ArticleListSetAllUnread) => {
                self.model_data.set_all_read_status(Read::Unread)?;
                view_needs_update = true;
            }

            Message::Event(AsyncMarkArticlesAsReadFinished) => {
                model_needs_update = true;
            }

            Message::Command(ArticleListSelectNextUnread) => {
                self.select_next_unread()?;
            }

            Message::Command(ArticleListSearch(query)) => {
                *self.filter_state.article_search_query_mut() = Some(query.clone());
                self.view_data.update(
                    self.config.clone(),
                    &self.model_data,
                    &self.filter_state,
                    self.is_focused,
                ); // manual here for highlighting only
                self.search_next(false, false)?;
            }

            Message::Command(ArticleListSearchNext) => {
                self.search_next(true, false)?;
            }

            Message::Command(ArticleListSearchPrevious) => {
                self.search_next(true, true)?;
            }

            Message::Command(ArticleListFilterSet(article_adhoc_filter)) => {
                self.filter_state
                    .on_new_article_adhoc_filter(article_adhoc_filter.clone());
                model_needs_update = true;
            }

            Message::Command(ArticleListFilterApply) => {
                *self.filter_state.apply_article_adhoc_filter_mut() = true;
                model_needs_update = true;
            }

            Message::Command(ArticleListFilterClear) => {
                *self.filter_state.apply_article_adhoc_filter_mut() = false;
                model_needs_update = true;
            }

            _ => {}
        }

        // update state where needed
        if model_needs_update {
            self.model_data.update(&self.filter_state).await?;
        }

        if model_needs_update || view_needs_update {
            self.view_data.update(
                self.config.clone(),
                &self.model_data,
                &self.filter_state,
                self.is_focused,
            );
            self.restore_sensible_selection(current_article.as_ref())?;
        }

        Ok(())
    }
}
