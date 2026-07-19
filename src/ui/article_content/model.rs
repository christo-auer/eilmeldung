use crate::prelude::*;

use std::{
    io::Cursor,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};

use getset::Getters;
use htmd::HtmlToMarkdown;
use image::ImageReader;
use news_flash::models::{Article, ArticleID, Enclosure, FatArticle, Feed, Tag, Thumbnail};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

#[derive(Getters)]
#[getset(get = "pub(super)")]
pub struct ArticleContentModelData {
    #[getset(skip)]
    news_flash_utils: Arc<NewsFlashUtils>,

    // Core article data
    article: Option<Article>,
    feed: Option<Feed>,
    tags: Option<Vec<Tag>>,
    fat_article: Option<FatArticle>,
    enclosures: Option<Vec<Enclosure>>,

    // Processed content
    markdown_content: Option<String>,

    // Filtered content
    filtered_markdown_content: Option<String>,

    // Thumbnail data and state
    thumbnail_fetch_successful: Option<bool>,
    thumbnail_fetch_running: bool,
    thumbnail: Option<Thumbnail>,

    // Timing for debouncing fetches
    instant_since_article_selected: Option<Instant>,
    duration_since_last_article_change: Option<Duration>,
}

impl ArticleContentModelData {
    pub(super) fn new(news_flash_utils: Arc<NewsFlashUtils>) -> Self {
        Self {
            news_flash_utils,

            article: None,
            feed: None,
            tags: None,
            fat_article: None,
            markdown_content: None,
            filtered_markdown_content: None,
            thumbnail_fetch_successful: None,
            thumbnail_fetch_running: false,
            thumbnail: None,
            instant_since_article_selected: None,
            duration_since_last_article_change: None,
            enclosures: None,
        }
    }

    pub(super) async fn on_article_selected(
        &mut self,
        article_id: &ArticleID,
    ) -> color_eyre::Result<bool> {
        if self.article.as_ref().map(|article| &article.article_id) == Some(article_id) {
            return Ok(false);
        }

        let current_instant = Instant::now();
        if let Some(last_article_selected) = self.instant_since_article_selected {
            self.duration_since_last_article_change =
                Some(current_instant.duration_since(last_article_selected));
        }
        self.instant_since_article_selected = Some(current_instant);
        self.thumbnail_fetch_successful = None;
        self.thumbnail = None;
        self.fat_article = None;
        self.markdown_content = None;
        self.filtered_markdown_content = None;
        self.feed = None;
        self.tags = None;

        let news_flash = self.news_flash_utils.news_flash_lock.read().await;

        let article = news_flash.get_article(article_id)?;

        self.feed = news_flash
            .get_feeds()?
            .0
            .into_iter()
            .find(|feed| feed.feed_id == article.feed_id);

        let (tags, taggings) = news_flash.get_tags()?;
        let mut tag_for_tag_id = NewsFlashUtils::generate_id_map(&tags, |tag| tag.tag_id.clone());
        self.tags = Some(
            taggings
                .into_iter()
                .filter(|tagging| tagging.article_id == article.article_id)
                .filter_map(|tagging| tag_for_tag_id.remove(&tagging.tag_id))
                .collect::<Vec<Tag>>(),
        );

        self.enclosures = Some(news_flash.get_enclosures(article_id)?);

        self.article = Some(article);

        Ok(true)
    }

    pub(super) fn prepare_thumbnail(
        &mut self,
        thumbnail: &Thumbnail,
        picker: &Picker,
    ) -> color_eyre::Result<Option<StatefulProtocol>> {
        if let Some(article) = self.article.as_ref()
            && article.article_id == thumbnail.article_id
            && let Some(data) = thumbnail.data.as_ref()
        {
            self.thumbnail = Some(thumbnail.to_owned());
            Ok(Some(Self::gen_stateful_protocol(data, picker)?))
        } else {
            Ok(None)
        }
    }

    pub(super) fn update_thumbnail(
        &self,
        picker: &Picker,
    ) -> color_eyre::Result<Option<StatefulProtocol>> {
        if let Some(thumbnail) = self.thumbnail.as_ref()
            && let Some(data) = thumbnail.data.as_ref()
        {
            Ok(Some(Self::gen_stateful_protocol(data, picker)?))
        } else {
            Ok(None)
        }
    }

    fn gen_stateful_protocol(data: &[u8], picker: &Picker) -> color_eyre::Result<StatefulProtocol> {
        let cursor = Cursor::new(data);
        let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
        Ok(picker.new_resize_protocol(image))
    }

    pub(super) fn scrape_article(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Ok(());
        };

        if self.fat_article.is_none() {
            let article_id = article.article_id.clone();
            self.news_flash_utils.fetch_fat_article(article_id);
        }

        Ok(())
    }

    pub(super) fn set_fat_article(&mut self, fat_article: FatArticle) {
        self.fat_article = Some(fat_article);
        self.markdown_content = None; // Reset processed content
    }

    pub(super) fn get_or_create_markdown_content(
        &mut self,
        config: &Config,
    ) -> color_eyre::Result<()> {
        if self.markdown_content.is_none()
            && config.content_preferred_type == ArticleContentType::Markdown
            && let Some(fat_article) = self.fat_article.as_ref()
            && let Some(html) = fat_article.scraped_content.as_deref()
        {
            // create html to markdown converter
            let html2markdown = HtmlToMarkdown::builder().build();

            self.markdown_content = Some(html2markdown.convert(html)?);
        }
        Ok(())
    }

    pub(super) fn update_should_fetch_thumbnail(&mut self, config: &Config) -> bool {
        if !config.thumbnail_show || self.thumbnail_fetch_running {
            return false;
        }

        let Some(article) = self.article.as_ref() else {
            return false;
        };

        if article.thumbnail_url.is_none() {
            self.thumbnail_fetch_successful = Some(false);
            return false;
        }

        if !self.thumbnail_fetch_successful.unwrap_or(true) {
            return false;
        }

        let current_instant = Instant::now();
        let long_enough_current_article = match self.instant_since_article_selected {
            Some(article_selected_instant) => {
                let duration = current_instant.duration_since(article_selected_instant);
                duration >= Duration::from_millis(config.thumbnail_fetch_debounce_millis)
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

    pub(super) fn start_fetch_thumbnail(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            self.thumbnail_fetch_successful = Some(false);
            return Ok(());
        };

        if article.thumbnail_url.is_none() {
            self.thumbnail_fetch_successful = Some(false);
            return Ok(());
        }

        let article_id = article.article_id.clone();
        self.news_flash_utils.fetch_thumbnail(article_id);
        self.thumbnail_fetch_running = true;

        Ok(())
    }

    pub(super) fn on_thumbnail_fetch_finished(&mut self, thumbnail: Option<&Thumbnail>) {
        self.thumbnail_fetch_running = false;
        match thumbnail {
            Some(_) => {
                self.thumbnail_fetch_successful = Some(true);
            }
            None => {
                log::debug!("fetching thumbnail not successful");
                self.thumbnail_fetch_successful = Some(false);
            }
        }
    }

    pub(super) fn on_thumbnail_fetch_failed(&mut self) {
        self.thumbnail_fetch_successful = Some(false);
        self.thumbnail_fetch_running = false;
    }

    pub(super) fn clean_string(string: &str) -> String {
        string.replace("\r", "").replace("\n", "")
    }

    pub(crate) async fn open_enclosure(
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

    pub(crate) fn pipe(
        &self,
        config: &Config,
        in_target: PipeTarget,
        out_target: PipeTarget,
        command: &str,
    ) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Err(color_eyre::eyre::eyre!("no article selected"));
        };

        let command = command
            .replace(
                "{date}",
                &article
                    .date
                    .with_timezone(&chrono::Local)
                    .format(&config.date_format)
                    .to_string(),
            )
            .replace("{title}", article.title.as_deref().unwrap_or_default())
            .replace(
                "{url}",
                article
                    .url
                    .as_ref()
                    .map(|url| url.as_str())
                    .unwrap_or_default(),
            )
            .replace("{author}", article.author.as_deref().unwrap_or_default())
            .replace(
                "{feed}",
                self.feed
                    .as_ref()
                    .map(|feed| &*feed.label)
                    .unwrap_or_default(),
            );

        self.news_flash_utils.pipe(
            article.to_owned(),
            self.fat_article.to_owned(),
            in_target,
            out_target,
            command.to_string(),
        );

        Ok(())
    }

    pub(crate) fn on_pipe_finished(
        &mut self,
        article_id: &ArticleID,
        exit_status: std::process::ExitStatus,
        markdown: &Option<String>,
        error: &Option<String>,
    ) {
        if self.article.as_ref().map(|article| &article.article_id) != Some(article_id) {
            return;
        }

        if markdown.is_none()
            && exit_status.success()
            && error.as_ref().map(|error| error.is_empty()).unwrap_or(true)
        {
            return;
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

        self.filtered_markdown_content.replace(markdown_content);
    }
}
