mod model;
mod view;

pub mod prelude {
    pub use super::ArticlesList;
}

use crate::ui::articles_list::{model::ArticleListModelData, view::FilterState};
use log::info;
use news_flash::models::{Article, ArticleID, Marked, Read, Tag};
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
            filter_state: FilterState::new(config.article_scope, config.default_sort_order.clone()),

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
                    article.article_id.to_owned(),
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

    pub(super) fn open_in_browser(&self, action_scope: &ActionScope) -> color_eyre::Result<()> {
        let articles = self.get_articles_by_action_scope(action_scope)?;

        for article in articles.iter() {
            if let Some(url) = article.url.as_ref()
                && let Err(error) = webbrowser::open(url.as_ref())
            {
                tooltip(
                    &self.message_sender,
                    format!("unable to open article in webbrowser: {}", error).as_str(),
                    TooltipFlavor::Error,
                )?;
                break;
            }
        }

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
                self.model_data.last_sync(),
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

        let scrollbar_state = self.view_data.scrollbar_state_mut();
        *scrollbar_state = scrollbar_state.position(index);

        let Some(lines) = *self.view_data.article_lines() else {
            return;
        };
        let offset = self.view_data.get_table_state_mut().offset_mut();
        let max_lines_above =
            (lines as usize).saturating_sub(self.config.articles_after_selection + 1);

        if index.saturating_sub(*offset) > max_lines_above {
            *offset = index.saturating_sub(max_lines_above);
        }
    }

    pub(super) fn get_articles_by_action_scope(
        &self,
        action_scope: &ActionScope,
    ) -> color_eyre::Result<Vec<Article>> {
        use ActionScope::*;
        Ok(match action_scope {
            All => self.model_data.articles().clone(),
            Current => self.get_current_article().iter().cloned().collect(),
            Query(query) => self.model_data.get_queried_articles(query).to_vec(),
        })
    }

    pub(super) fn get_article_ids_by_action_scope(
        &self,
        action_scope: &ActionScope,
    ) -> color_eyre::Result<Vec<ArticleID>> {
        Ok(self
            .get_articles_by_action_scope(action_scope)?
            .into_iter()
            .map(|article| article.article_id)
            .collect())
    }

    pub(super) fn set_action_scope_read_status(
        &mut self,
        action_scope: &ActionScope,
        read: Read,
    ) -> color_eyre::Result<usize> {
        let amount = self
            .model_data
            .set_read_status(self.get_article_ids_by_action_scope(action_scope)?, read)?;

        tooltip(
            &self.message_sender,
            format!(
                "set status of {} articles to {}",
                amount,
                match read {
                    Read::Read => "read",
                    Read::Unread => "read",
                }
            )
            .as_str(),
            TooltipFlavor::Info,
        )?;

        Ok(amount)
    }

    pub(super) fn set_action_scope_marked_status(
        &mut self,
        action_scope: &ActionScope,
        marked: Marked,
    ) -> color_eyre::Result<usize> {
        let amount = self
            .model_data
            .set_marked_status(self.get_article_ids_by_action_scope(action_scope)?, marked)?;

        tooltip(
            &self.message_sender,
            format!(
                "set status of {} articles to {}",
                amount,
                match marked {
                    Marked::Marked => "marked",
                    Marked::Unmarked => "unmarked",
                }
            )
            .as_str(),
            TooltipFlavor::Info,
        )?;

        Ok(amount)
    }

    pub(super) fn change_tag(
        &mut self,
        action_scope: &ActionScope,
        tag: Tag,
        add: bool,
    ) -> color_eyre::Result<()> {
        let article_ids = self.get_article_ids_by_action_scope(action_scope)?;
        if add {
            let amount = self
                .model_data
                .tag_articles(article_ids, tag.tag_id.clone())?;
            tooltip(
                &self.message_sender,
                format!("adding tag {} to {} articles", tag.label, amount).as_str(),
                TooltipFlavor::Info,
            )?;
            amount
        } else {
            let amount = self
                .model_data
                .untag_articles(article_ids, tag.tag_id.clone())?;
            tooltip(
                &self.message_sender,
                format!("removing tag {} from {} articles", tag.label, amount).as_str(),
                TooltipFlavor::Info,
            )?;
            amount
        };

        Ok(())
    }

    pub(super) fn search_next(
        &mut self,
        skip_current: bool,
        reversed: bool,
    ) -> color_eyre::Result<()> {
        let offset = if skip_current { 1 } else { 0 };
        let Some(article_query) = self.filter_state.article_search_query().as_ref() else {
            return tooltip(
                &self.message_sender,
                "no search query",
                TooltipFlavor::Warning,
            );
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

    fn target_matches(&self, target: &ActionTarget) -> bool {
        match target {
            ActionTarget::Current if self.is_focused => true,
            ActionTarget::ArticleList => true,
            _ => false,
        }
    }

    fn on_tag_or_untag(
        &mut self,
        action_scope: ActionScope,
        tag_name: String,
        tag_articles: bool,
    ) -> Result<(), color_eyre::eyre::Error> {
        match self
            .model_data
            .tag_map()
            .values()
            .find(|&tag| tag.label == tag_name)
        {
            Some(tag) => {
                self.change_tag(&action_scope, tag.clone(), tag_articles)?;
            }
            None => {
                log::warn!("could not find tag with name {}", tag_name);
                tooltip(
                    &self.message_sender,
                    format!("unknown tag: {}", tag_name).as_str(),
                    TooltipFlavor::Error,
                )?;
            }
        }
        Ok(())
    }
}

impl crate::messages::MessageReceiver for ArticlesList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        if matches!(message, Message::Event(Event::Tick)) {
            return Ok(());
        }

        let current_article = self.get_current_article().map(|article| article.article_id);
        let mut model_needs_update = false;
        let mut view_needs_update = false;

        if let Message::Command(command) = message {
            use Command as C;
            let mut handle_command = false;

            let Some(command) = (match command {
                C::In(Panel::ArticleList, command) => {
                    handle_command = true;
                    Some(*command.to_owned())
                }
                C::In(..) => None,
                command => {
                    handle_command = self.is_focused;
                    Some(command.to_owned())
                }
            }) else {
                return Ok(());
            };

            match command {
                C::NavigateUp if handle_command => {
                    self.view_data.get_table_state_mut().select_previous();
                    self.select_index_and_send_message(None)?;
                }
                C::NavigateDown if handle_command => {
                    self.view_data.get_table_state_mut().select_next();
                    self.select_index_and_send_message(None)?;
                }
                C::NavigatePageUp if handle_command => {
                    let lines = *self.view_data.article_lines();
                    if let Some(lines) = lines {
                        self.view_data
                            .get_table_state_mut()
                            .scroll_up_by(lines.saturating_sub(1));
                        self.select_index_and_send_message(None)?;
                    };
                }
                C::NavigatePageDown if handle_command => {
                    let lines = *self.view_data.article_lines();
                    if let Some(lines) = lines {
                        self.view_data.get_table_state_mut().scroll_down_by(
                            lines.saturating_sub(self.config.articles_after_selection as u16),
                        );
                        self.select_index_and_send_message(None)?;
                    }
                }
                C::NavigateFirst if handle_command => {
                    self.view_data.get_table_state_mut().select_first();
                    self.select_index_and_send_message(None)?;
                }
                C::NavigateLast if handle_command => {
                    self.view_data.get_table_state_mut().select_last();
                    // manually "select" as select_last does not know the number of rows
                    self.select_index_and_send_message(Some(self.model_data.articles().len() - 1))?;
                }

                show_command @ (C::Show(ActionTarget::ArticleList, scope)
                | C::Show(ActionTarget::Current, scope))
                    if matches!(show_command, C::Show(ActionTarget::ArticleList, _))
                        || handle_command =>
                {
                    *self.filter_state.article_scope_mut() = scope;
                    model_needs_update = true;
                }

                // actions
                C::ActionOpenInBrowser(action_scope) => {
                    self.open_in_browser(&action_scope)?;
                }

                C::ActionSetRead(target, action_scope) if self.target_matches(&target) => {
                    // first we check if we can reroute this to feed list to make it quicker
                    use ActionScope::*;
                    match action_scope {
                        // queries must be handled by article list
                        Query(_) => {
                            self.set_action_scope_read_status(&action_scope, Read::Read)?;
                            view_needs_update = true;
                        }

                        // if all articles must be set to read and no adhoc filter is applied, the
                        // feed list can set the articles to read
                        All if !self.filter_state.apply_article_adhoc_filter()
                            && !matches!(target, ActionTarget::ArticleList) =>
                        {
                            info!("re-routing set read command to feed list");
                            self.message_sender.send(Message::Command(C::ActionSetRead(
                                ActionTarget::FeedList,
                                ActionScope::Current,
                            )))?;
                        }

                        _ => {
                            self.set_action_scope_read_status(&action_scope, Read::Read)?;
                            view_needs_update = true;
                        }
                    }
                }

                C::ActionSetUnread(action_scope) => {
                    self.set_action_scope_read_status(&action_scope, Read::Unread)?;
                    view_needs_update = true;
                }

                C::ActionSetMarked(action_scope) => {
                    self.set_action_scope_marked_status(&action_scope, Marked::Marked)?;
                    view_needs_update = true;
                }

                C::ActionSetUnmarked(action_scope) => {
                    self.set_action_scope_marked_status(&action_scope, Marked::Unmarked)?;
                    view_needs_update = true;
                }

                C::ActionTagArticles(action_scope, tag_name) => {
                    self.on_tag_or_untag(action_scope, tag_name, true)?;
                    view_needs_update = true;
                }

                C::ActionUntagArticles(action_scope, tag_name) => {
                    self.on_tag_or_untag(action_scope, tag_name, false)?;
                    view_needs_update = true;
                }

                C::InputFind if handle_command => {
                    self.message_sender
                        .send(Message::Command(C::CommandLineOpen(Some(
                            "search".to_owned(),
                        ))))?;
                }

                C::ArticleListSelectNextUnread => {
                    self.select_next_unread()?;
                }

                C::ArticleListSearch(query) => {
                    *self.filter_state.article_search_query_mut() = Some(query.clone());
                    self.view_data.update(
                        self.config.clone(),
                        &self.model_data,
                        &self.filter_state,
                        handle_command,
                    ); // manual here for highlighting only
                    self.search_next(false, false)?;
                }

                C::ArticleListSearchNext => {
                    self.search_next(true, false)?;
                }

                C::ArticleListSearchPrevious => {
                    self.search_next(true, true)?;
                }

                C::ArticleListFilterSet(article_adhoc_filter) => {
                    self.filter_state
                        .on_new_article_adhoc_filter(article_adhoc_filter.clone());
                    model_needs_update = true;
                }

                C::ArticleListFilterApply => {
                    *self.filter_state.apply_article_adhoc_filter_mut() = true;
                    model_needs_update = true;
                }

                C::ArticleListFilterClear => {
                    *self.filter_state.apply_article_adhoc_filter_mut() = false;
                    model_needs_update = true;
                }

                C::ArticleListSort(sort_order) => {
                    *self.filter_state.adhoc_sort_order_mut() = Some(sort_order.to_owned());
                    model_needs_update = true;
                }

                C::ArticleListSortReverse => {
                    *self.filter_state.reverse_sort_order_mut() =
                        !*self.filter_state.reverse_sort_order_mut();
                    model_needs_update = true;
                }

                C::ArticleListSortClear => {
                    self.filter_state.clear_sort_order();
                    model_needs_update = true;
                }

                C::ArticleListQuery(query) => {
                    self.message_sender
                        .send(Message::Event(Event::ArticlesSelected(query.into())))?;
                }
                _ => {}
            }
        }

        if let Message::Event(event) = message {
            use Event::*;
            match event {
                ArticlesSelected(augmented_article_filter) => {
                    self.filter_state
                        .on_new_article_filter(augmented_article_filter.clone());
                    model_needs_update = true;
                }

                ApplicationStateChanged(state) => {
                    self.is_focused = *state == AppState::ArticleSelection;
                    view_needs_update = true;
                }

                event if event.caused_model_update() => model_needs_update = true,

                _ => {}
            }
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
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
