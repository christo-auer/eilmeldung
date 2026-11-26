mod markdown_to_text;
mod model;
mod state;
mod view;

pub mod prelude {
    pub use super::ArticleContent;
}

use model::ArticleContentModelData;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;
use view::ArticleContentViewData;

use crate::{
    prelude::*,
    ui::article_content::{state::LoadedArticle, view::HintTarget},
};
use std::{process::Stdio, sync::Arc};

use news_flash::models::{ArticleID, Enclosure};

#[derive(getset::CopyGetters)]
pub struct ArticleContent {
    config: Arc<Config>,

    view_data: ArticleContentViewData,
    model_data: ArticleContentModelData,

    message_sender: UnboundedSender<Message>,

    #[getset(get_copy = "pub")]
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
            view_data: ArticleContentViewData::new(Arc::clone(&news_flash_utils), &config),
            config,
            model_data: ArticleContentModelData::new(news_flash_utils),
            message_sender,
            is_focused: false,
            is_distraction_free: false,
        }
    }

    async fn on_article_selected(
        &mut self,
        article_id: Option<&ArticleID>,
    ) -> color_eyre::Result<ModelState> {
        Ok(if let Some(article_id) = article_id {
            self.model_data.on_article_selected(article_id).await?
        } else {
            self.model_data.on_no_article_selected()
        })
    }

    fn share_url(&self, target_str: &str, title: &str, url: &str) -> color_eyre::Result<()> {
        let Some(target) = self
            .config
            .share_targets
            .iter()
            .find(|target| target.as_ref() == target_str)
        else {
            tooltip(
                &self.message_sender,
                &*format!("unknown share target {target_str}"),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

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

    fn share_article(&self, target_str: &str) -> color_eyre::Result<()> {
        let Some(LoadedArticle { article, .. }) = self.model_data.loaded_article().as_ref() else {
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

        self.share_url(target_str, title, url.as_ref())?;

        Ok(())
    }

    fn pipe(
        &self,
        in_target: PipeTarget,
        out_target: PipeTarget,
        command: &str,
    ) -> color_eyre::Result<ModelState> {
        let dirty = self
            .model_data
            .on_pipe(&self.config, in_target, out_target, command);

        tooltip(
            &self.message_sender,
            &*Command::Pipe(in_target, out_target, command.into()).to_string(),
            TooltipFlavor::Info,
        )?;

        Ok(dirty)
    }

    fn on_pipe_finished(
        &mut self,
        article_id: &ArticleID,
        exit_status: std::process::ExitStatus,
        markdown: &Option<String>,
        error: &Option<String>,
    ) -> color_eyre::Result<ModelState> {
        Ok(self
            .model_data
            .on_pipe_finished(article_id, exit_status, markdown, error))
    }

    async fn open_enclosure(
        &self,
        enclosure_type: Option<EnclosureType>,
    ) -> color_eyre::Result<()> {
        let Some(matching_enclosure) = self.model_data.get_matching_enclosure(enclosure_type)
        else {
            tooltip(
                &self.message_sender,
                "no matching enclosure found",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        match self.execute_enclosure_command(&self.config, matching_enclosure) {
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

    fn execute_enclosure_command(
        &self,
        config: &Config,
        enclosure: &Enclosure,
    ) -> color_eyre::Result<String> {
        let command = match <EnclosureType>::from(enclosure) {
            EnclosureType::Audio => config.audio_enclosure_command.as_ref(),
            EnclosureType::Image => config.image_enclosure_command.as_ref(),
            EnclosureType::Video => config.video_enclosure_command.as_ref(),
        }
        .unwrap_or(&config.enclosure_command)
        .replace("{url}", enclosure.url.as_ref())
        .replace("{type}", <EnclosureType>::from(enclosure).as_ref())
        .replace("{mime}", enclosure.mime_type.as_deref().unwrap_or("*/*"));

        let (cmd, args) = prepare_command(&command)?;

        let mut command = std::process::Command::new(&cmd);

        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(args)
            .spawn()?;

        Ok(cmd.to_owned())
    }

    fn open_hint(&mut self, command: &Command) -> color_eyre::Result<()> {
        let hint = match command {
            Command::ContentFollowHint(hint) => &hint.to_owned(),
            Command::ContentShareHint(_, hint) => &hint.to_owned(),
            Command::ContentImageLoad(ImageLoadArg::Hint(hint)) => &hint.to_owned(),
            _ => unreachable!(),
        };

        let Some(hint_target) = self.view_data.resolve_hint(&hint.to_uppercase()).cloned() else {
            tooltip(&self.message_sender, "unknown hint", TooltipFlavor::Error)?;
            return Ok(());
        };

        let url = hint_target.url();
        let url_result = Url::parse(url);

        let Ok(_parsed_url) = url_result else {
            tooltip(
                &self.message_sender,
                &*format!("cannot open invalid URL: {}", url_result.unwrap_err()),
                TooltipFlavor::Error,
            )?;
            return Ok(());
        };

        let title = self
            .model_data
            .loaded_article()
            .as_ref()
            .and_then(|loaded_article| loaded_article.article.title.as_ref())
            .map(|title| format!("Link from {title}"))
            .unwrap_or("".to_owned());

        match command {
            Command::ContentFollowHint(..) => {
                if let Err(err) = webbrowser::open(url) {
                    tooltip(
                        &self.message_sender,
                        &*format!("unable to open webbrowser: {err}"),
                        TooltipFlavor::Error,
                    )?;
                }
            }
            Command::ContentShareHint(target_str, ..) => {
                self.share_url(target_str, &title, url)?;
            }
            Command::ContentImageLoad(..) => {
                if !matches!(hint_target, HintTarget::Image(..)) {
                    tooltip(
                        &self.message_sender,
                        "this hint does not point to an image",
                        TooltipFlavor::Error,
                    )?;
                } else {
                    self.view_data.load_image(url);
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_command(&mut self, command: &Command) -> color_eyre::Result<ModelState> {
        use Command as C;
        use ModelState as S;
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
            return Ok(ModelState::Clean);
        };

        Ok(match command {
            C::NavigateDown(amount) if handle_command => {
                self.view_data.scroll_down(amount.unwrap_or(1));
                S::Clean
            }
            C::NavigateUp(amount) if handle_command => {
                self.view_data.scroll_up(amount.unwrap_or(1));
                S::Clean
            }
            C::NavigatePageUp if handle_command => {
                self.view_data
                    .scroll_page_up(self.config.input_config.scroll_amount as u16);
                S::Clean
            }
            C::NavigatePageDown if handle_command => {
                self.view_data
                    .scroll_page_down(self.config.input_config.scroll_amount as u16);
                S::Clean
            }
            C::NavigateFirst if handle_command => {
                self.view_data.scroll_to_top();
                S::Clean
            }
            C::NavigateLast if handle_command => {
                self.view_data.scroll_to_bottom();
                S::Clean
            }

            C::ActionScrape(ActionScope::Current) if handle_command => {
                self.model_data.scrape_article()?
            }

            C::ArticleShare(target) => {
                self.share_article(&target)?;
                S::Clean
            }

            C::Pipe(in_target, out_target, command) => {
                self.pipe(in_target, out_target, &command)?
            }

            C::Refresh => S::Dirty,

            C::ArticleOpenEnclosure(enclosure_type) => {
                self.open_enclosure(enclosure_type).await?;
                S::Clean
            }

            hint_command @ (C::ContentFollowHint(..)
            | C::ContentShareHint(..)
            | C::ContentImageLoad(ImageLoadArg::Hint(_))) => {
                self.open_hint(&hint_command)?;
                S::Clean
            }

            C::ContentImageLoad(ImageLoadArg::Visible) => {
                self.view_data.load_visible_images();
                S::Clean
            }

            // C::Clear => {
            //     TODO
            //     self.view_data.clear(&self.config);
            // }
            set_read_command @ C::ActionSetRead(_) if handle_command => {
                // don't know what to do with this -> re-route to article list
                self.message_sender.send(Message::Command(C::In(
                    Panel::ArticleList,
                    Box::new(set_read_command),
                )))?;
                S::Clean
            }

            _ => S::Clean,
        })
    }

    async fn on_event(&mut self, event: &Event) -> color_eyre::Result<ModelState> {
        use Event as E;
        use ModelState as S;
        Ok(match event {
            E::ArticleSelected(article_id) => {
                let model_state = self.on_article_selected(article_id.as_ref()).await?;
                if model_state.is_dirty() && self.is_focused && self.config.auto_scrape {
                    log::debug!("auto-scraping article");
                    model_state.and_then(self.model_data.scrape_article()?)
                } else {
                    log::debug!("not auto-scraping article");
                    model_state
                }
            }

            E::AsyncArticleThumbnailFetchFinished(thumbnail) => {
                log::debug!("fetching thumbnail successful");
                self.model_data
                    .on_thumbnail_fetch_finished(thumbnail.to_owned())
            }

            E::AsyncImageFetchFailed(url, async_operation_error) => {
                log::error!("{async_operation_error}");
                self.view_data.on_image_fetch_failed(url)
            }

            E::AsyncImageFetchFinished(url, image) => self
                .view_data
                .on_image_fetch_finished(url, image.to_owned()),

            E::AsyncOperationFailed(err, reason) => {
                if let Event::AsyncArticleThumbnailFetch = *reason.as_ref() {
                    log::debug!("fetching thumbnail not successful: {err}");
                    self.model_data.on_thumbnail_fetch_failed()
                } else {
                    S::Clean
                }
            }

            E::AsyncPipeArticleFinished(article_id, exit_status, markdown, error) => {
                self.on_pipe_finished(article_id, *exit_status, markdown, error)?
            }

            E::AsyncArticleFatFetchFinished(fat_article) => {
                self.model_data.on_fat_article(fat_article.clone())
            }

            E::ApplicationStateChanged(state) => {
                self.is_focused = *state == AppState::ArticleContent
                    || *state == AppState::ArticleContentDistractionFree;

                self.is_distraction_free = *state == AppState::ArticleContentDistractionFree;

                if self.is_focused && self.config.auto_scrape {
                    self.model_data.scrape_article()?;
                }
                S::Dirty
            }

            E::AsyncArticleTagFinished | E::AsyncArticleUntagFinished => {
                self.model_data.update_article_tags().await?
            }

            E::Tick => self
                .view_data
                .tick(&self.model_data)
                .and_then(self.model_data.on_update_thumbnail_state(&self.config)),

            E::ImageProtocolPickerUpdated(picker) => {
                log::trace!("image protocol picker updated");
                self.view_data.picker_updated(picker, &self.config)
            }

            E::ConfigReloaded(config) => {
                self.config = Arc::clone(config);
                // self.view_data.clear(&self.config); TODO
                S::Dirty
            }

            _ => {
                if event.caused_model_update() {
                    S::Dirty
                } else {
                    S::Clean
                }
            }
        })
    }
}

impl crate::messages::MessageReceiver for ArticleContent {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        let model_state = match message {
            Message::Command(command) => self.on_command(command).await?,
            Message::Event(event) => self.on_event(event).await?,
            _ => ModelState::Clean,
        };

        if model_state.is_dirty() {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
