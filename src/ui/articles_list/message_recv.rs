use crate::prelude::*;
use std::collections::HashMap;

use news_flash::models::{Article, ArticleID, Feed, FeedID, Read, Tag, TagID};

#[derive(Default)]
pub struct ArticleListModelData {
    pub(super) articles: Vec<Article>,
    pub(super) feed_map: HashMap<FeedID, Feed>,
    pub(super) tags_for_article: HashMap<ArticleID, Vec<TagID>>,
    pub(super) tag_map: HashMap<TagID, Tag>,
}

impl ArticlesList {
    pub(super) async fn update_model_data(&mut self) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        // fill model data
        let (feeds, _) = news_flash.get_feeds()?;
        self.model_data.feed_map = NewsFlashUtils::generate_id_map(&feeds, |f| f.feed_id.clone());

        let (tags, taggings) = news_flash.get_tags()?;
        self.model_data.tag_map = NewsFlashUtils::generate_id_map(&tags, |t| t.tag_id.clone());

        self.model_data.tags_for_article = NewsFlashUtils::generate_one_to_many(
            &taggings,
            |a| a.article_id.clone(),
            |t| t.tag_id.clone(),
        );

        let position_for_tag = tags
            .iter()
            .enumerate()
            .map(|(pos, tag)| (&tag.tag_id, pos))
            .collect::<HashMap<&TagID, usize>>();

        self.model_data
            .tags_for_article
            .iter_mut()
            .for_each(|(_, tag_ids)| {
                tag_ids.sort_by(|tag_a, tag_b| {
                    position_for_tag
                        .get(tag_a)
                        .unwrap()
                        .cmp(position_for_tag.get(tag_b).unwrap())
                })
            });

        drop(news_flash);

        // apply the current filter
        let news_flash_utils = self.news_flash_utils.clone();
        self.view_data
            .filter_state
            .fill_articles(news_flash_utils, &mut self.model_data)
            .await
    }

    pub(super) async fn set_all_read_status(&mut self, read: Read) -> color_eyre::Result<()> {
        let article_ids: Vec<ArticleID> = self
            .model_data
            .articles
            .iter()
            .map(|article| article.article_id.clone())
            .collect();

        self.news_flash_utils.set_article_status(article_ids, read);

        self.model_data
            .articles
            .iter_mut()
            .for_each(|article| article.unread = read);

        Ok(())
    }

    pub(super) async fn set_current_read_status(
        &mut self,
        read: Option<Read>,
    ) -> color_eyre::Result<()> {
        let news_flash_utils = self.news_flash_utils.clone();
        if let Some(article) = self.get_current_article_mut() {
            let new_state = match read {
                Some(state) => state,
                None => article.unread.invert(),
            };

            let article_id = article.article_id.clone();

            news_flash_utils.set_article_status(vec![article_id], new_state);

            // update locally
            article.unread = new_state;
        }

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
            Message::Command(NavigateUp) if self.view_data.is_focused => {
                self.view_data.table_state.select_previous();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateDown) if self.view_data.is_focused => {
                self.view_data.table_state.select_next();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigatePageUp) if self.view_data.is_focused => {
                self.view_data
                    .table_state
                    .scroll_up_by(self.config.theme.articles_list_height_lines - 1);
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigatePageDown) if self.view_data.is_focused => {
                self.view_data.table_state.scroll_down_by(
                    self.config.theme.articles_list_height_lines
                        - self.config.articles_list_visible_articles_after_selection as u16,
                );
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateFirst) if self.view_data.is_focused => {
                self.view_data.table_state.select_first();
                self.select_index_and_send_message(None)?;
            }
            Message::Command(NavigateLast) if self.view_data.is_focused => {
                self.view_data.table_state.select_last();
                // manually "select" as select_last does not know the number of rows
                self.select_index_and_send_message(Some(self.model_data.articles.len() - 1))?;
            }

            Message::Event(AsyncOperationFailed(_, _)) => {
                model_needs_update = true;
            }

            Message::Event(ArticlesSelected(augmented_article_filter)) => {
                self.view_data
                    .filter_state
                    .on_new_article_filter(augmented_article_filter.clone());
                model_needs_update = true;
            }

            Message::Command(ArticleListSetScope(scope)) => {
                self.view_data.filter_state.article_scope = *scope;
                model_needs_update = true;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.view_data.is_focused = *state == AppState::ArticleSelection;
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
                self.set_all_read_status(Read::Read).await?;
                view_needs_update = true;
            }

            Message::Command(ArticleListSetAllUnread) => {
                self.set_all_read_status(Read::Unread).await?;
                view_needs_update = true;
            }

            Message::Event(AsyncMarkArticlesAsReadFinished) => {
                model_needs_update = true;
            }

            Message::Command(ArticleListSelectNextUnread) => {
                self.select_next_unread()?;
            }

            Message::Command(ArticleListSearch(query)) => {
                self.view_data.filter_state.article_search_query = Some(query.clone());
                self.update_view_data(); // manual here for highlighting only
                self.search_next(false, false)?;
            }

            Message::Command(ArticleListSearchNext) => {
                self.search_next(true, false)?;
            }

            Message::Command(ArticleListSearchPrevious) => {
                self.search_next(true, true)?;
            }

            Message::Command(ArticleListFilterSet(article_adhoc_filter)) => {
                self.view_data
                    .filter_state
                    .on_new_article_adhoc_filter(article_adhoc_filter.clone());
                model_needs_update = true;
            }

            Message::Command(ArticleListFilterApply) => {
                self.view_data.filter_state.apply_article_adhoc_filter = true;
                model_needs_update = true;
            }

            Message::Command(ArticleListFilterClear) => {
                self.view_data.filter_state.apply_article_adhoc_filter = false;
                model_needs_update = true;
            }

            _ => {}
        }

        // update state where needed
        if model_needs_update {
            self.update_model_data().await?;
        }

        if model_needs_update || view_needs_update {
            self.update_view_data();
            self.restore_sensible_selection(current_article.as_ref())?;
        }

        Ok(())
    }
}
