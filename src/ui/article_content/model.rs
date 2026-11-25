use crate::{
    prelude::*,
    ui::article_content::state::{ArticleBody, LoadedArticle, ThumbnailState},
};

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use getset::Getters;
use html_to_markdown_rs::ConversionOptions;
use itertools::Itertools;
use jiff::tz::TimeZone;
use news_flash::models::{Article, ArticleID, Enclosure, FatArticle, Thumbnail};

#[derive(Getters)]
#[getset(get = "pub(super)")]
pub struct ArticleContentModelData {
    #[getset(skip)]
    news_flash_utils: Arc<NewsFlashUtils>,

    // Core article data
    loaded_article: Option<LoadedArticle>,

    // Timing for debouncing fetches
    instant_since_article_selected: Option<Instant>,
    duration_since_last_article_change: Option<Duration>,
}

impl ArticleContentModelData {
    pub(super) fn new(news_flash_utils: Arc<NewsFlashUtils>) -> Self {
        Self {
            news_flash_utils,
            loaded_article: Default::default(),
            instant_since_article_selected: None,
            duration_since_last_article_change: None,
        }
    }

    pub(super) fn on_no_article_selected(&mut self) -> ModelState {
        self.loaded_article = None;
        ModelState::Dirty
    }

    pub(super) async fn on_article_selected(
        &mut self,
        article_id: &ArticleID,
    ) -> color_eyre::Result<ModelState> {
        log::debug!("article selected: {article_id:?}");

        if self
            .loaded_article
            .as_ref()
            .map(|content| &content.article().article_id)
            == Some(article_id)
        {
            log::debug!("article id has not changed");
            return Ok(ModelState::Clean);
        }

        self.thumbnail_fetch_debounce_reset();

        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        let Ok(article) = news_flash.get_article(article_id).await else {
            return Ok(ModelState::Dirty);
        };

        let feed = news_flash
            .get_feeds()
            .await?
            .0
            .into_iter()
            .find(|feed| feed.feed_id == article.feed_id)
            .ok_or(color_eyre::eyre::eyre!(
                "unable to find feed for article id"
            ))?;

        let enclosures = news_flash.get_enclosures(article_id).await?;
        let tags = news_flash.get_tags_of_article(article_id).await?;

        let mut article_content = LoadedArticle {
            article,
            feed,
            tags,
            enclosures,
            thumbnail_state: ThumbnailState::None,
            body: Default::default(),
        };

        if let Ok(fat_article) = news_flash.get_fat_article(article_id).await
            && let Some(markdown_content) = Self::fat_article_to_markdown(&fat_article)
        {
            article_content.body = ArticleBody::Fat {
                fat_article,
                markdown_content,
            };
        }

        self.loaded_article = Some(article_content);

        Ok(ModelState::Dirty)
    }

    fn fat_article_to_markdown(fat_article: &FatArticle) -> Option<String> {
        let html = fat_article.scraped_content.as_deref()?;

        html_to_markdown_rs::convert(html, ConversionOptions::default())
            .ok()
            .and_then(|result| result.content)
    }

    pub(super) async fn update_article_tags(&mut self) -> color_eyre::Result<ModelState> {
        if let Some(LoadedArticle {
            article: Article { article_id, .. },
            tags,
            ..
        }) = self.loaded_article.as_mut()
        {
            let news_flash = self.news_flash_utils.news_flash_lock.read().await;
            *tags = news_flash.get_tags_of_article(article_id).await?;
            return Ok(ModelState::Dirty);
        }
        Ok(ModelState::Clean)
    }

    pub(super) fn scrape_article(&mut self) -> color_eyre::Result<ModelState> {
        let Some(loaded_article) = self.loaded_article.as_ref() else {
            return Ok(ModelState::Clean);
        };

        self.news_flash_utils
            .fetch_fat_article(loaded_article.article_id().to_owned());

        Ok(ModelState::Clean)
    }

    pub(super) fn on_fat_article(&mut self, fat_article: FatArticle) -> ModelState {
        let Some(loaded_article) = self.loaded_article.as_mut() else {
            log::debug!("received fat article but there is no article loaded");
            return ModelState::Clean;
        };

        if loaded_article.article_id() != &fat_article.article_id {
            log::debug!(
                "set_fat_article: article id {:?} does not match fat article id {:?}",
                loaded_article.article_id(),
                fat_article.article_id
            );
            return ModelState::Clean;
        }

        match loaded_article.body {
            ArticleBody::Summary => {
                if let Some(markdown_content) = Self::fat_article_to_markdown(&fat_article) {
                    loaded_article.body = ArticleBody::Fat {
                        fat_article,
                        markdown_content,
                    };
                    ModelState::Dirty
                } else {
                    log::debug!("fat article does not contain scraped data");
                    ModelState::Clean
                }
            }
            _ => {
                log::debug!(
                    "received fat article again => ignoring {fat_article:?} vs {:?}",
                    loaded_article.body
                );
                ModelState::Clean
            }
        }
    }

    pub(super) fn on_update_thumbnail_state(&mut self, config: &Config) -> ModelState {
        if !config.thumbnail_show {
            return ModelState::Clean; // don't deal with thumbnails at all
        }

        use ThumbnailState as S;
        let debounce_check = self.thumbnail_fetch_debounce_check(config);

        let Some(loaded_article) = self.loaded_article.as_mut() else {
            return ModelState::Clean;
        };

        if !debounce_check {
            return ModelState::Clean;
        }

        if matches!(loaded_article.thumbnail_state, S::None) {
            self.news_flash_utils
                .fetch_thumbnail(loaded_article.article_id().to_owned());
            loaded_article.thumbnail_state = S::Fetching;
            return ModelState::Dirty;
        }

        ModelState::Clean
    }

    fn thumbnail_fetch_debounce_reset(&mut self) {
        let current_instant = Instant::now();

        if let Some(last_article_selected) = self.instant_since_article_selected {
            self.duration_since_last_article_change = Some(last_article_selected.elapsed());
        }

        self.instant_since_article_selected = Some(current_instant);
    }

    fn thumbnail_fetch_debounce_check(&mut self, config: &Config) -> bool {
        let long_enough_current_article = match self.instant_since_article_selected {
            Some(article_selected_instant) => {
                article_selected_instant.elapsed()
                    >= Duration::from_millis(config.thumbnail_fetch_debounce_millis)
            }
            None => true,
        };

        match self.duration_since_last_article_change {
            None => true,
            Some(duration) => {
                duration > Duration::from_millis(config.thumbnail_fetch_debounce_millis)
                    || long_enough_current_article
            }
        }
    }

    pub(super) fn on_thumbnail_fetch_finished(
        &mut self,
        article_id: &ArticleID,
        thumbnail: Option<Thumbnail>,
    ) -> ModelState {
        if let Some(loaded_article) = self.loaded_article.as_mut()
            && loaded_article.article_id() == article_id
        {
            loaded_article.thumbnail_state = match thumbnail {
                Some(thumbnail) => ThumbnailState::Fetched(thumbnail),
                None => ThumbnailState::Failed,
            };

            ModelState::Dirty
        } else {
            ModelState::Clean
        }
    }

    pub(super) fn on_thumbnail_fetch_failed(&mut self, article_id: &ArticleID) -> ModelState {
        if let Some(loaded_article) = self.loaded_article.as_mut()
            && loaded_article.article_id() == article_id
        {
            loaded_article.thumbnail_state = ThumbnailState::Failed;
            ModelState::Dirty
        } else {
            ModelState::Clean
        }
    }

    pub(super) fn get_matching_enclosure(
        &self,
        enclosure_type: Option<EnclosureType>,
    ) -> Option<&Enclosure> {
        let LoadedArticle { enclosures, .. } = self.loaded_article.as_ref()?;

        let enclosures_matching_type = enclosures
            .iter()
            .filter(|enclosure| {
                enclosure_type
                    .map(|enclosure_type| enclosure_type == (*enclosure).into())
                    .unwrap_or(true)
            })
            .collect::<Vec<&Enclosure>>();

        enclosures_matching_type
            .into_iter()
            .find_or_first(|enclosure| enclosure.is_default)
    }

    pub(crate) fn on_pipe(
        &self,
        config: &Config,
        in_target: PipeTarget,
        out_target: PipeTarget,
        command: &str,
    ) -> ModelState {
        let Some(LoadedArticle {
            article,
            feed,
            body,
            ..
        }) = self.loaded_article.as_ref()
        else {
            return ModelState::Clean;
        };

        let command = command
            .replace(
                "{date}",
                &article
                    .date
                    .to_zoned_datetime(TimeZone::system())
                    .strftime(&config.date_format)
                    .to_string(),
            )
            .replace("{title}", article.title.as_deref().unwrap_or_default())
            .replace("{url}", article.url.as_deref().unwrap_or_default())
            .replace("{author}", article.author.as_deref().unwrap_or_default())
            .replace("{feed}", &feed.label);

        let fat_article = body.fat_article().cloned();

        self.news_flash_utils.pipe(
            article.to_owned(),
            fat_article,
            in_target,
            out_target,
            command.to_string(),
        );

        ModelState::Clean
    }

    pub(crate) fn on_pipe_finished(
        &mut self,
        article_id: &ArticleID,
        exit_status: std::process::ExitStatus,
        markdown: &Option<String>,
        error: &Option<String>,
    ) -> ModelState {
        let Some(loaded_article) = self.loaded_article.as_mut() else {
            log::debug!("received pipe output but don't have article anymore");
            return ModelState::Clean;
        };

        if !loaded_article.is(article_id) {
            log::debug!("received pipe output for different article");
            return ModelState::Clean;
        }

        if markdown.is_none()
            && exit_status.success()
            && error.as_ref().map(|error| error.is_empty()).unwrap_or(true)
        {
            return ModelState::Clean;
        }

        let mut markdown_content = markdown.as_deref().unwrap_or("").to_string();

        if !exit_status.success() {
            markdown_content.push_str(&format!(
                r#"
---

Process returned with error ({})
                "#,
                exit_status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown error code".to_string())
            ));
        }

        if let Some(error) = error.as_deref() {
            markdown_content.push_str(&format!(
                r#"
---
 
# stderr output

{error}
                "#
            ));
        }

        loaded_article.body = ArticleBody::Piped {
            piped_content: markdown_content,
        };

        ModelState::Dirty
    }
}
