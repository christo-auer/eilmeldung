mod model;
mod view;

use model::ArticleContentModelData;
use view::ArticleContentViewData;

use crate::prelude::*;
use std::sync::Arc;

use news_flash::models::Thumbnail;
use tokio::sync::mpsc::UnboundedSender;

pub struct ArticleContent {
    config: Arc<Config>,

    view_data: ArticleContentViewData,
    model_data: ArticleContentModelData,

    is_focused: bool,
}

impl ArticleContent {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            view_data: ArticleContentViewData::default(),
            model_data: ArticleContentModelData::new(news_flash_utils, message_sender.clone()),
            is_focused: false,
        }
    }

    async fn on_article_selected(
        &mut self,
        article: &news_flash::models::Article,
        feed: &Option<news_flash::models::Feed>,
        tags: &Option<Vec<news_flash::models::Tag>>,
    ) -> color_eyre::Result<()> {
        self.model_data
            .on_article_selected(article, feed, tags, self.is_focused)
            .await?;
        self.view_data.clear_image();
        self.view_data.scroll_to_top();
        self.view_data.update(&self.model_data, self.config.clone());
        Ok(())
    }

    fn prepare_thumbnail(&mut self, thumbnail: &Thumbnail) -> color_eyre::Result<()> {
        let image = self
            .model_data
            .prepare_thumbnail(thumbnail, self.view_data.picker())?;
        self.view_data.set_image(image);
        Ok(())
    }

    fn scrape_article(&mut self) -> color_eyre::Result<()> {
        self.model_data.scrape_article(self.is_focused)?;
        // Reset scroll when new content is loaded
        if self.model_data.fat_article().is_some() {
            *self.view_data.vertical_scroll_mut() = 0;
        }
        Ok(())
    }

    fn tick(&mut self) -> color_eyre::Result<()> {
        self.view_data.tick_throbber();

        if self.model_data.should_fetch_thumbnail(&self.config) {
            self.fetch_thumbnail()?;
        }
        Ok(())
    }

    fn fetch_thumbnail(&mut self) -> color_eyre::Result<()> {
        if self.view_data.image().is_none() {
            self.model_data.fetch_thumbnail()?;
            self.view_data.reset_thumbnail_throbber();
        }
        Ok(())
    }
}

impl crate::messages::MessageReceiver for ArticleContent {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        let mut view_needs_update = false;

        match message {
            Message::Command(NavigateDown) if self.is_focused => {
                self.view_data.scroll_down();
            }
            Message::Command(NavigateUp) if self.is_focused => {
                self.view_data.scroll_up();
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.view_data
                    .scroll_page_up(self.config.input_config.scroll_amount as u16);
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.view_data
                    .scroll_page_down(self.config.input_config.scroll_amount as u16);
            }
            Message::Command(NavigateFirst) if self.is_focused => {
                self.view_data.scroll_to_top();
            }
            Message::Command(NavigateLast) if self.is_focused => {
                self.view_data.scroll_to_bottom();
            }

            // TODO fetch article data directly instead of using command
            Message::Event(ArticleSelected(article, feed, tags)) => {
                self.on_article_selected(article, feed, tags).await?;
            }

            Message::Event(FatArticleSelected(article, feed, tags)) => {
                self.model_data
                    .on_article_selected(article, feed, tags, self.is_focused)
                    .await?;

                if self.is_focused && self.config.article_auto_scrape {
                    self.scrape_article()?;
                }
                view_needs_update = true;
            }

            Message::Event(AsyncFetchThumbnailFinished(thumbnail)) => {
                self.model_data
                    .on_thumbnail_fetch_finished(thumbnail.as_ref());
                match thumbnail {
                    Some(thumbnail) => {
                        self.prepare_thumbnail(thumbnail)?;
                    }
                    None => {
                        log::debug!("fetching thumbnail not successful");
                        self.view_data.clear_image();
                        self.model_data.on_thumbnail_fetch_failed();
                    }
                }
                view_needs_update = true;
            }

            Message::Event(AsyncOperationFailed(err, reason)) => {
                if let Event::AsyncFetchThumbnail = *reason.as_ref() {
                    log::debug!("fetching thumbnail not successful: {err}");
                    self.view_data.clear_image();
                    self.model_data.on_thumbnail_fetch_failed();
                    view_needs_update = true;
                }
            }

            Message::Command(Command::ArticleCurrentScrape) if self.is_focused => {
                self.scrape_article()?;
            }

            Message::Event(AsyncFetchFatArticleFinished(fat_article)) => {
                self.model_data.set_fat_article(fat_article.clone());
                // Process markdown content if needed
                self.model_data.get_or_create_markdown_content(&self.config);
                view_needs_update = true;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::ArticleContent
                    || *state == AppState::ArticleContentDistractionFree;

                if self.is_focused && self.config.article_auto_scrape {
                    self.scrape_article()?;
                }
                view_needs_update = true;
            }

            Message::Event(AsyncSyncFinished(_))
            | Message::Event(AsyncMarkArticlesAsReadFinished)
            | Message::Event(AsyncMarkArticlesAsMarkedFinished)
            | Message::Event(AsyncTagArticleFinished) => {
                view_needs_update = true;
            }

            Message::Event(Tick) => {
                self.tick()?;
            }

            _ => {}
        }

        if view_needs_update {
            self.view_data.update(&self.model_data, self.config.clone());
        }

        Ok(())
    }
}
