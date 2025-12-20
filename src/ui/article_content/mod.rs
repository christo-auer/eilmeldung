mod model;
mod view;

pub mod prelude {
    pub use super::ArticleContent;
}

use arboard::Clipboard;
use model::ArticleContentModelData;
use url::Url;
use view::ArticleContentViewData;

use crate::prelude::*;
use std::sync::Arc;

use news_flash::models::{ArticleID, Thumbnail};
use tokio::sync::mpsc::UnboundedSender;

pub struct ArticleContent {
    config: Arc<Config>,

    view_data: ArticleContentViewData,
    model_data: ArticleContentModelData,

    message_sender: UnboundedSender<Message>,

    is_focused: bool,
    is_distraction_free: bool,
}

impl ArticleContent {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config,
            view_data: ArticleContentViewData::default(),
            model_data: ArticleContentModelData::new(news_flash_utils, message_sender.clone()),
            message_sender,
            is_focused: false,
            is_distraction_free: false,
        }
    }

    async fn on_article_selected(&mut self, article_id: &ArticleID) -> color_eyre::Result<()> {
        self.model_data
            .on_article_selected(article_id, self.is_focused)
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

    fn share_article(&self, target_str: &String) -> color_eyre::Result<()> {
        let Some(target) = self
            .config
            .share_targets
            .iter()
            .find(|target| target.as_ref() == *target_str)
        else {
            tooltip(
                &self.message_sender,
                &*format!("unknown share target {target_str}"),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

        let Some(article) = self.model_data.article() else {
            tooltip(
                &self.message_sender,
                "no article loaded",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        let Some(url) = article.url.as_ref() else {
            tooltip(
                &self.message_sender,
                "article has no URL",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        let title: &str = article.title.as_deref().unwrap_or("no title");
        let url: &Url = url.as_ref();

        match target {
            ShareTarget::Clipboard => {
                let mut clipboard = Clipboard::new()?;
                clipboard.set_text(url.to_string())?;
                tooltip(
                    &self.message_sender,
                    &*format!("copied URL to clipboard ({url})"),
                    TooltipFlavor::Info,
                )?;
                Ok(())
            }

            target => {
                let share_url = target.to_url(title, url)?;
                webbrowser::open(share_url.to_string().as_str())?;
                tooltip(
                    &self.message_sender,
                    format!("shared article to {target}").as_str(),
                    TooltipFlavor::Info,
                )?;
                Ok(())
            }
        }
    }
}

impl crate::messages::MessageReceiver for ArticleContent {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut view_needs_update = false;

        if let Message::Command(command) = message {
            use Command as C;
            match command {
                C::NavigateDown if self.is_focused => {
                    self.view_data.scroll_down();
                }
                C::NavigateUp if self.is_focused => {
                    self.view_data.scroll_up();
                }
                C::NavigatePageUp if self.is_focused => {
                    self.view_data
                        .scroll_page_up(self.config.input_config.scroll_amount as u16);
                }
                C::NavigatePageDown if self.is_focused => {
                    self.view_data
                        .scroll_page_down(self.config.input_config.scroll_amount as u16);
                }
                C::NavigateFirst if self.is_focused => {
                    self.view_data.scroll_to_top();
                }
                C::NavigateLast if self.is_focused => {
                    self.view_data.scroll_to_bottom();
                }

                C::ArticleCurrentScrape if self.is_focused => {
                    self.scrape_article()?;
                }

                C::ArticleShare(target) => {
                    self.share_article(target)?;
                }

                _ => {}
            }
        }

        if let Message::Event(event) = message {
            use Event::*;
            match event {
                ArticleSelected(article_id) => {
                    self.on_article_selected(article_id).await?;
                    view_needs_update = true;
                }

                FatArticleSelected(article) => {
                    self.model_data
                        .on_article_selected(article, self.is_focused)
                        .await?;

                    if self.is_focused && self.config.auto_scrape {
                        self.scrape_article()?;
                    }
                    view_needs_update = true;
                }

                AsyncArticleThumbnailFetchFinished(thumbnail) => {
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

                AsyncOperationFailed(err, reason) => {
                    if let Event::AsyncArticleThumbnailFetch = *reason.as_ref() {
                        log::debug!("fetching thumbnail not successful: {err}");
                        self.view_data.clear_image();
                        self.model_data.on_thumbnail_fetch_failed();
                        view_needs_update = true;
                    }
                }

                AsyncArticleFatFetchFinished(fat_article) => {
                    self.model_data.set_fat_article(fat_article.clone());
                    // Process markdown content if needed
                    self.model_data.get_or_create_markdown_content(&self.config);
                    view_needs_update = true;
                }

                ApplicationStateChanged(state) => {
                    self.is_focused = *state == AppState::ArticleContent
                        || *state == AppState::ArticleContentDistractionFree;

                    self.is_distraction_free = *state == AppState::ArticleContentDistractionFree;

                    if self.is_focused && self.config.auto_scrape {
                        self.scrape_article()?;
                    }

                    view_needs_update = true;
                }

                Tick => {
                    self.tick()?;
                }

                event if event.caused_model_update() => {
                    view_needs_update = true;
                }

                _ => {}
            }
        }

        if view_needs_update {
            self.view_data.update(&self.model_data, self.config.clone());
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
