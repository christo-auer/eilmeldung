use crate::prelude::*;

use std::{
    io::Cursor,
    sync::Arc,
    time::{Duration, Instant},
};

use getset::Getters;
use image::ImageReader;
use news_flash::{
    models::{Article, ArticleID, FatArticle, Feed, Tag, Thumbnail},
    util::html2text,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Getters)]
#[getset(get = "pub(super)")]
pub struct ArticleContentModelData {
    #[getset(skip)]
    news_flash_utils: Arc<NewsFlashUtils>,

    #[getset(skip)]
    message_sender: UnboundedSender<Message>,

    // Core article data
    article: Option<Article>,
    feed: Option<Feed>,
    tags: Option<Vec<Tag>>,
    fat_article: Option<FatArticle>,

    // Processed content
    markdown_content: Option<String>,

    // Thumbnail data and state
    thumbnail_fetch_successful: Option<bool>,
    thumbnail_fetch_running: bool,

    // Timing for debouncing fetches
    instant_since_article_selected: Option<Instant>,
    duration_since_last_article_change: Option<Duration>,
}

impl ArticleContentModelData {
    pub(super) fn new(
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            news_flash_utils,
            message_sender,

            article: None,
            feed: None,
            tags: None,
            fat_article: None,
            markdown_content: None,
            thumbnail_fetch_successful: None,
            thumbnail_fetch_running: false,
            instant_since_article_selected: None,
            duration_since_last_article_change: None,
        }
    }

    pub(super) async fn on_article_selected(
        &mut self,
        article_id: &ArticleID,
        is_focused: bool,
    ) -> color_eyre::Result<()> {
        if let Some(current_article) = self.article.as_ref()
            && current_article.article_id == *article_id
        {
            return Ok(());
        }

        let current_instant = Instant::now();
        if let Some(last_article_selected) = self.instant_since_article_selected {
            self.duration_since_last_article_change =
                Some(current_instant.duration_since(last_article_selected));
        }
        self.instant_since_article_selected = Some(current_instant);
        self.thumbnail_fetch_successful = None;
        self.fat_article = None;
        self.markdown_content = None;
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

        self.article = Some(article);

        if is_focused {
            self.message_sender
                .send(Message::Event(Event::FatArticleSelected(
                    article_id.clone(),
                )))?;
        }

        Ok(())
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
            let cursor = Cursor::new(data);
            let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
            Ok(Some(picker.new_resize_protocol(image)))
        } else {
            Ok(None)
        }
    }

    pub(super) fn scrape_article(&mut self, is_focused: bool) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Ok(());
        };

        if is_focused && self.fat_article.is_none() {
            let article_id = article.article_id.clone();
            self.news_flash_utils.fetch_fat_article(article_id);
        }

        Ok(())
    }

    pub(super) fn set_fat_article(&mut self, fat_article: FatArticle) {
        self.fat_article = Some(fat_article);
        self.markdown_content = None; // Reset processed content
    }

    pub(super) fn get_or_create_markdown_content(&mut self, config: &Config) -> Option<&str> {
        if self.markdown_content.is_none() {
            if config.article_content_preferred_type == ArticleContentType::Markdown {
                if let Some(fat_article) = self.fat_article.as_ref() {
                    if let Some(html) = fat_article.scraped_content.as_deref() {
                        self.markdown_content = Some(html2text::html2text(html));
                    }
                }
            }
        }
        self.markdown_content.as_deref()
    }

    pub(super) fn should_fetch_thumbnail(&self, config: &Config) -> bool {
        if !config.article_thumbnail_show || self.thumbnail_fetch_running {
            return false;
        }

        let Some(article) = self.article.as_ref() else {
            return false;
        };

        if article.thumbnail_url.is_none() {
            return false;
        }

        if !self.thumbnail_fetch_successful.unwrap_or(true) {
            return false;
        }

        let current_instant = Instant::now();
        let long_enough_current_article = match self.instant_since_article_selected {
            Some(article_selected_instant) => {
                let duration = current_instant.duration_since(article_selected_instant);
                duration >= Duration::from_millis(config.article_thumbnail_fetch_debounce_millis)
            }
            None => true,
        };

        let long_enough = match self.duration_since_last_article_change {
            None => true,
            Some(duration) => {
                duration > Duration::from_millis(config.article_thumbnail_fetch_debounce_millis)
                    || long_enough_current_article
            }
        };

        long_enough
    }

    pub(super) fn fetch_thumbnail(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
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
}
