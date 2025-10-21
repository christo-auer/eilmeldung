use std::{
    io::Cursor,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    app::AppState,
    commands::{Command, Event, Message, MessageReceiver},
    config::{ArticleContentType, Config},
    newsflash_utils::NewsFlashUtils,
};
use image::ImageReader;
use news_flash::{
    models::{Article, FatArticle, Feed, Thumbnail},
    util::html2text,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_image::{
    FilterType, Resize, StatefulImage, picker::Picker, protocol::StatefulProtocol,
};

use tokio::sync::mpsc::UnboundedSender;

pub struct ArticleContent {
    config: Arc<Config>,

        news_flash_async_manager: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    article: Option<Article>,
    feed: Option<Feed>,
    fat_article: Option<FatArticle>,
    thumbnail: Option<Thumbnail>,
    image: Option<StatefulProtocol>,
    markdown_content: Option<String>,
    picker: Picker,

    vertical_scroll: u16,
    max_scroll: u16,

    is_focused: bool,

    instant_since_article_selected: Option<Instant>,
    duration_since_last_article_change: Option<Duration>,
}

impl ArticleContent {
    pub fn new(
        config: Arc<Config>,
    news_flash_async_manager: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config,
            message_sender,
            news_flash_async_manager: news_flash_async_manager.clone(),

            fat_article: None,
            article: None,
            feed: None,
            thumbnail: None,
            image: None,
            markdown_content: None,
            picker: Picker::from_query_stdio().unwrap(), // TODO gracefully handle errors

            vertical_scroll: 0,
            max_scroll: 0,

            is_focused: false,

            instant_since_article_selected: None,
            duration_since_last_article_change: None,
        }
    }

    async fn on_article_selected(&mut self, article: Article) -> color_eyre::Result<()> {
        let current_instant = Instant::now();
        if let Some(last_article_selected) = self.instant_since_article_selected {
            self.duration_since_last_article_change =
                Some(current_instant.duration_since(last_article_selected));
        }
        self.instant_since_article_selected = Some(current_instant);
        self.thumbnail = None;
        self.image = None;
        self.fat_article = None;
        self.markdown_content = None;
        self.feed = None;

        self.article = Some(article.clone());

        {
            let news_flash = self.news_flash_async_manager.news_flash_lock.read().await;

            let (feeds, _) = news_flash.get_feeds()?;

            self.feed = feeds
                .iter()
                .find(|feed| feed.feed_id == article.feed_id)
                .cloned();
        }

        if self.is_focused {
            self.message_sender
                .send(Message::Event(Event::FatArticleSelected(article.clone())))?;
        }

        Ok(())
    }

    fn prepare_thumbnail(&mut self, thumbnail: &Thumbnail) -> color_eyre::Result<()> {
        if let Some(article) = self.article.clone()
            && article.article_id == thumbnail.article_id
            && let Some(data) = thumbnail.data.as_ref()
        {
            let cursor = Cursor::new(data);
            let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
            self.image = Some(self.picker.new_resize_protocol(image));
        }

        Ok(())
    }

    fn scrape_article(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Ok(());
        };

        if self.is_focused && self.fat_article.is_none() {
            let article_id = article.article_id.clone();
            self.vertical_scroll = 0;
            self.news_flash_async_manager.fetch_fat_article(article_id);
        }

        Ok(())
    }

    fn render_block(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let mut block = Block::default()
            .borders(Borders::all())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(self.config.theme.border_style);

        if self.is_focused {
            block = block.title_bottom(
                Line::from(format!(
                    " {}% ",
                    f64::round((self.vertical_scroll as f64 / self.max_scroll as f64) * 100.0)
                        as u16
                ))
                .right_aligned(),
            )
        }

        let inner_area = block.inner(area);
        block.render(area, buf);
        inner_area
    }

    fn generate_summary<'a>(&'a self) -> Vec<Line<'a>> {
        let article = self.article.as_ref().unwrap();
        let title = article.title.clone().unwrap_or("no title".into());
        let feed_label: String = if let Some(feed) = self.feed.clone() {
            feed.label.clone()
        } else {
            article.feed_id.as_str().into()
        };
        let mut summary = article.summary.clone().unwrap_or("no summary".into());
        summary = ArticleContent::clean_string(&mut summary);

        vec![
            Line::from(Span::from(feed_label).style(self.config.theme.feed)),
            Line::from(Span::from(title).style(self.config.theme.header)),
            Line::from(Span::from(summary).style(self.config.theme.paragraph)),
        ]
    }

    fn render_summary(&mut self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.render_block(area, buf);

        let thumbnail_width = if self.config.article_thumbnail_show {
            self.config.article_thumbnail_width
        } else {
            0
        };

        let [thumbnail_chunk, summary_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages([
                thumbnail_width,
                100 - thumbnail_width,
            ]))
            .margin(1)
            .spacing(1)
            .areas(inner_area);

        if self.config.article_thumbnail_show
            && let Some(image) = &mut self.image
        {
            let mut stateful_image = StatefulImage::new();
            if self.config.article_thumbnail_resize {
                stateful_image = stateful_image.resize(Resize::Scale(Some(FilterType::Lanczos3)))
            }
            stateful_image.render(thumbnail_chunk, buf, image);
        }

        let paragraph = Paragraph::new(self.generate_summary()).wrap(Wrap { trim: true });

        paragraph.render(summary_chunk, buf);
    }

    fn render_fat_article(&mut self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.render_block(area, buf);

        let [paragraph_area] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(ratatui::layout::Flex::Center)
            .constraints([
                Constraint::Max(self.config.article_content_max_chars_per_line), // Middle content - maximum 80 columns
            ])
            .areas(inner_area);

        let Some(fat_article) = self.fat_article.as_ref() else {
            return;
        };

        let text: Text<'_> = if self.config.article_content_preferred_type
            == ArticleContentType::Markdown
            // && self.markdown_content.is_none()
            && let Some(html) = fat_article.scraped_content.clone()
        {
            if self.markdown_content.is_none() {
                self.markdown_content = Some(html2text::html2text(html.as_str()));
            }

            tui_markdown::from_str(self.markdown_content.as_ref().unwrap())
        } else if let Some(plain_text) = fat_article.plain_text.clone() {
            Text::from(plain_text)
        } else {
            Text::from("no content available")
        };

        // Calculate the total number of lines the content would take when wrapped
        let content_lines = self.calculate_wrapped_lines(&text, paragraph_area.width);

        // Calculate maximum scroll (ensure it doesn't go negative)
        self.max_scroll = content_lines.saturating_sub(paragraph_area.height);

        // Ensure current scroll doesn't exceed maximum
        self.vertical_scroll = self.vertical_scroll.min(self.max_scroll);

        let content = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll((self.vertical_scroll, 0));

        content.render(paragraph_area, buf);
    }

    fn clean_string(string: &mut str) -> String {
        string.replace("\r", "").replace("\n", "")
    }

    fn calculate_wrapped_lines(&self, text: &ratatui::text::Text, width: u16) -> u16 {
        let mut total_lines = 0u16;

        for line in text.lines.iter() {
            if line.spans.is_empty() {
                total_lines += 1;
                continue;
            }

            let line_content: String = line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect();

            if line_content.is_empty() {
                total_lines += 1;
            } else {
                // Calculate how many lines this content will take when wrapped
                let line_width = line_content.chars().count() as u16;
                let wrapped_lines = (line_width + width - 1) / width.max(1); // Ceiling division
                total_lines += wrapped_lines.max(1);
            }
        }

        total_lines
    }

    fn tick(&mut self) -> color_eyre::Result<()> {
        if self.config.article_thumbnail_show {
            self.fetch_thumbnail()?;
        }
        Ok(())
    }

    fn fetch_thumbnail(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Ok(());
        };

        if article.thumbnail_url.is_none() {
            return Ok(());
        }

        let current_instant = Instant::now();
        let long_enough_current_article = match self.instant_since_article_selected {
            Some(article_selected_instant) => {
                let duration = current_instant.duration_since(article_selected_instant);
                duration
                    >= Duration::from_millis(self.config.article_thumbnail_fetch_debounce_millis)
            }
            None => true,
        };

        let long_enough = match self.duration_since_last_article_change {
            None => true,
            Some(duration) => {
                duration
                    > Duration::from_millis(self.config.article_thumbnail_fetch_debounce_millis)
                    || long_enough_current_article
            }
        };

        if !long_enough {
            return Ok(());
        }

        if self.image.is_none() {
            let article_id = article.article_id.clone();
            self.news_flash_async_manager.fetch_thumbnail(article_id);
        }

        Ok(())
    }
}

impl Widget for &mut ArticleContent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if !self.is_focused && self.article.is_some() {
            self.render_summary(area, buf);
        }

        if self.is_focused {
            if self.fat_article.is_some() {
                self.render_fat_article(area, buf);
            } else if self.article.is_some() {
                self.render_summary(area, buf);
            }
        }
    }
}

impl MessageReceiver for ArticleContent {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;
        match message {
            Message::Command(NavigateDown) if self.is_focused => {
                self.vertical_scroll = (self.vertical_scroll + 1).min(self.max_scroll)
            }
            Message::Command(NavigateUp) if self.is_focused => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1)
            }
            Message::Command(NavigatePageUp) if self.is_focused => {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_sub(self.config.input_config.scroll_amount as u16)
            }
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.vertical_scroll = (self.vertical_scroll
                    + self.config.input_config.scroll_amount as u16)
                    .min(self.max_scroll)
            }

            Message::Command(NavigateFirst) if self.is_focused => {
                self.vertical_scroll = 0;
            }

            Message::Command(NavigateLast) if self.is_focused => {
                self.vertical_scroll = self.max_scroll;
            }

            Message::Event(ArticleSelected(article)) => {
                self.on_article_selected(article.clone()).await?;
            }

            Message::Event(FatArticleSelected(article)) => {
                self.article = Some(article.clone());

                if self.is_focused && self.config.article_auto_scrape {
                    self.scrape_article()?;
                }
            }

            Message::Event(AsyncFetchThumbnailFinished(thumbnail)) => match thumbnail {
                Some(thumbnail) => {
                    self.prepare_thumbnail(thumbnail)?;
                }
                None => {
                    self.image = None;
                }
            },

            Message::Command(Command::ArticleCurrentScrape) if self.is_focused => {
                self.scrape_article()?;
            }

            Message::Event(AsyncFetchFatArticleFinished(fat_article)) => {
                self.fat_article = Some(fat_article.clone());
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::ArticleContent
                    || *state == AppState::ArticleContentDistractionFree;

                if self.is_focused && self.config.article_auto_scrape {
                    self.scrape_article()?;
                }
            }

            Message::Event(Tick) => {
                self.tick()?;
            }

            _ => {}
        }

        Ok(())
    }
}
