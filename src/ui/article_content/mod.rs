mod model;
mod view;

pub mod prelude {
    pub use super::ArticleContent;
}

use model::ArticleContentModelData;
use url::Url;
use view::ArticleContentViewData;

use crate::prelude::*;
use std::sync::Arc;

use news_flash::models::{ArticleID, Enclosure, Thumbnail};
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
        self.update_thumbnail_fetching_state()?;
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
        self.model_data.scrape_article()?;
        // Reset scroll when new content is loaded
        // if self.model_data.fat_article().is_some() {
        //     *self.view_data.vertical_scroll_mut() = 0;
        // }
        Ok(())
    }

    fn update_thumbnail_fetching_state(&mut self) -> color_eyre::Result<bool> {
        self.view_data.tick_throbber();
        if self.model_data.update_should_fetch_thumbnail(&self.config) {
            self.fetch_thumbnail()?;
        }

        Ok(*self.model_data.thumbnail_fetch_running())
    }

    fn fetch_thumbnail(&mut self) -> color_eyre::Result<()> {
        if self.view_data.image().is_none() {
            self.model_data.start_fetch_thumbnail()?;
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

        match target.share(title, url) {
            Ok(()) => tooltip(
                &self.message_sender,
                &*format!("shared with {}", target),
                TooltipFlavor::Info,
            )?,
            Err(error) => tooltip(
                &self.message_sender,
                &*format!("unable to shared with {}: {}", target, error),
                TooltipFlavor::Error,
            )?,
        }

        Ok(())
    }

    async fn open_enclosure(
        &self,
        enclosure_type: Option<EnclosureType>,
    ) -> color_eyre::Result<()> {
        let Some(enclosures) = self.model_data.enclosures() else {
            tooltip(
                &self.message_sender,
                "no enclosures available",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        let enclosures_matching_type = enclosures
            .iter()
            .filter(|enclosure| {
                enclosure_type
                    .map(|enclosure_type| enclosure_type == (*enclosure).into())
                    .unwrap_or(true)
            })
            .collect::<Vec<&Enclosure>>();

        let matching_enclosure = enclosures_matching_type
            .iter()
            .find(|enclosure| enclosure.is_default)
            .or_else(|| enclosures_matching_type.first());

        let Some(matching_enclosure) = matching_enclosure else {
            tooltip(
                &self.message_sender,
                "no matching enclosure found",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        match self
            .model_data
            .open_enclosure(&self.config, matching_enclosure)
            .await
        {
            Ok(cmd) => tooltip(
                &self.message_sender,
                &*format!("openend enclosure with {cmd}"),
                TooltipFlavor::Info,
            )?,
            Err(err) => {
                return tooltip(
                    &self.message_sender,
                    err.to_string().as_str(),
                    TooltipFlavor::Error,
                );
            }
        }

        Ok(())
    }
}

impl crate::messages::MessageReceiver for ArticleContent {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut view_needs_update = false;

        if let Message::Command(command) = message {
            use Command as C;
            view_needs_update = true;
            let mut handle_command = false;

            let Some(command) = (match command {
                C::In(Panel::ArticleContent, command) => {
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
                C::NavigateDown if handle_command => {
                    self.view_data.scroll_down();
                }
                C::NavigateUp if handle_command => {
                    self.view_data.scroll_up();
                }
                C::NavigatePageUp if handle_command => {
                    self.view_data
                        .scroll_page_up(self.config.input_config.scroll_amount as u16);
                }
                C::NavigatePageDown if handle_command => {
                    self.view_data
                        .scroll_page_down(self.config.input_config.scroll_amount as u16);
                }
                C::NavigateFirst if handle_command => {
                    self.view_data.scroll_to_top();
                }
                C::NavigateLast if handle_command => {
                    self.view_data.scroll_to_bottom();
                }

                C::ArticleCurrentScrape => {
                    self.scrape_article()?;
                }

                C::ArticleShare(target) => {
                    self.share_article(&target)?;
                }

                C::Refresh => {
                    view_needs_update = true;
                }

                C::ArticleOpenEnclosure(enclosure_type) => {
                    self.open_enclosure(enclosure_type).await?;
                }

                _ => {
                    view_needs_update = false;
                }
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
                    self.model_data
                        .get_or_create_markdown_content(&self.config)?;
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
                    view_needs_update = self.update_thumbnail_fetching_state()?;
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
