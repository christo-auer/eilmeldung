use std::{io::Cursor, sync::Arc};

use crate::{
    app::AppState,
    commands::{Command, CommandReceiver},
    config::{ArticleContentType, Config},
    newsflash_utils::NewsFlashAsyncManager,
};
use image::ImageReader;
use news_flash::{
    models::{Article, FatArticle, Thumbnail},
    util::html2text,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_image::{
    FilterType, Resize, StatefulImage, picker::Picker, protocol::StatefulProtocol,
};

use tokio::sync::mpsc::UnboundedSender;

pub struct ArticleContent {
    config: Arc<Config>,

    news_flash_async_manager: Arc<NewsFlashAsyncManager>,
    _command_sender: UnboundedSender<Command>,

    article: Option<Article>,
    fat_article: Option<FatArticle>,
    thumbnail: Option<Thumbnail>,
    image: Option<StatefulProtocol>,
    markdown_content: Option<String>,
    picker: Picker,

    vertical_scroll: u16,
    max_scroll: u16,

    is_focused: bool,
}

impl ArticleContent {
    pub fn new(
        config: Arc<Config>,
        news_flash_async_manager: Arc<NewsFlashAsyncManager>,
        command_sender: UnboundedSender<Command>,
    ) -> Self {
        Self {
            config,
            _command_sender: command_sender,
            news_flash_async_manager: news_flash_async_manager.clone(),

            fat_article: None,
            article: None,
            thumbnail: None,
            image: None,
            markdown_content: None,
            picker: Picker::from_query_stdio().unwrap(), // gracefully handle errors

            vertical_scroll: 0,
            max_scroll: 0,

            is_focused: false,
        }
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

    fn load_article_thumbnail(&mut self) -> color_eyre::Result<()> {
        let Some(article) = self.article.as_ref() else {
            return Ok(());
        };

        if self.image.is_none() {
            let article_id = article.article_id.clone();
            self.news_flash_async_manager.fetch_thumbnail(article_id);
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

        let article = self.article.as_ref().unwrap();
        let mut summary = article.summary.clone().unwrap_or("no summary".into());
        summary = ArticleContent::clean_string(&mut summary);

        let summary = Paragraph::new(summary)
            .style(self.config.theme.paragraph)
            .wrap(Wrap { trim: true });

        summary.render(summary_chunk, buf);
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

impl CommandReceiver for ArticleContent {
    async fn process_command(&mut self, command: &Command) -> color_eyre::Result<()> {
        use Command::*;
        match command {
            NavigateDown => self.vertical_scroll = (self.vertical_scroll + 1).min(self.max_scroll),
            NavigateUp => self.vertical_scroll = self.vertical_scroll.saturating_sub(1),
            NavigatePageUp => {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_sub(self.config.input_config.scroll_amount as u16)
            }
            NavigatePageDown => {
                self.vertical_scroll = (self.vertical_scroll
                    + self.config.input_config.scroll_amount as u16)
                    .min(self.max_scroll)
            }

            ArticleSelected(article) => {
                self.thumbnail = None; // reset thumbnail
                self.image = None;
                self.fat_article = None;
                self.markdown_content = None;
                self.article = Some(article.clone());
            }

            FatArticleSelected(article) => self.article = Some(article.clone()),

            AsyncFetchThumbnailFinished(thumbnail) => match thumbnail {
                Some(thumbnail) => {
                    // TODO error handling
                    let data = thumbnail.data.as_ref().unwrap();
                    let cursor = Cursor::new(data);
                    let image = ImageReader::new(cursor).with_guessed_format()?.decode()?;
                    self.image = Some(self.picker.new_resize_protocol(image));
                }
                None => {
                    self.image = None;
                }
            },

            ScrapeArticle if self.is_focused => {
                self.scrape_article()?;
            }

            AsyncFetchFatArticleFinished(fat_article) => {
                self.fat_article = Some(fat_article.clone());
            }

            ApplicationStateChanged(state) => {
                self.is_focused = *state == AppState::ArticleContent;

                if self.config.article_auto_scrape {
                    self.scrape_article()?;
                }

                if self.config.article_thumbnail_show {
                    // TODO debounce
                    self.load_article_thumbnail()?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
