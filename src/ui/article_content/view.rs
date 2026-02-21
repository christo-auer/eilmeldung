use super::model::ArticleContentModelData;
use crate::prelude::*;

use std::{io::Cursor, sync::Arc};

use getset::{Getters, MutGetters};
use image::ImageReader;
use log::info;
use news_flash::models::Enclosure;
use ratatui::layout::Flex;
use ratatui_image::{
    FilterType, Resize, StatefulImage, picker::Picker, protocol::StatefulProtocol,
};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse};

const NO_THUMB_PLACEHOLDER: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/no-thumb.png"));

#[derive(Getters, MutGetters)]
pub struct ArticleContentViewData {
    // Scroll state
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    vertical_scroll: u16,
    #[getset(get = "pub(super)")]
    max_scroll: u16,

    #[getset(get = "pub(super), get_mut = "pub(super))]
    scrollbar_state: ScrollbarState,

    // Image rendering state
    image: Option<StatefulProtocol>,
    placeholder_image: StatefulProtocol,
    picker: Picker,

    // Throbber state for loading animations
    thumbnail_fetching_throbber: ThrobberState,
}

impl Default for ArticleContentViewData {
    fn default() -> Self {
        let picker = Picker::from_query_stdio().unwrap();
        let cursor = Cursor::new(NO_THUMB_PLACEHOLDER);
        let placeholder_image = picker.new_resize_protocol(
            ImageReader::new(cursor)
                .with_guessed_format()
                .unwrap() // OK as content is checked
                .decode()
                .unwrap(), // OK as content is checked
        );

        Self {
            vertical_scroll: 0,
            max_scroll: 0,
            image: None,
            placeholder_image,
            picker, // TODO gracefully handle errors
            thumbnail_fetching_throbber: ThrobberState::default(),
            scrollbar_state: ScrollbarState::default(),
        }
    }
}

impl ArticleContentViewData {
    pub(super) fn update(&mut self, model_data: &ArticleContentModelData, _config: Arc<Config>) {
        // Reset scroll when article changes
        if model_data.article().is_some() && self.vertical_scroll > self.max_scroll {
            self.vertical_scroll = 0;
        }
    }

    pub(super) fn set_image(&mut self, image: Option<StatefulProtocol>) {
        self.image = image;
    }

    pub(super) fn clear_image(&mut self) {
        self.image = None;
    }

    pub(super) fn reset_thumbnail_throbber(&mut self) {
        self.thumbnail_fetching_throbber.calc_next();
    }

    // Public accessors for private fields
    pub(super) fn picker(&self) -> &Picker {
        &self.picker
    }

    pub(super) fn image(&self) -> &Option<StatefulProtocol> {
        &self.image
    }

    pub(super) fn tick_throbber(&mut self) {
        self.thumbnail_fetching_throbber.calc_next();
    }

    pub(super) fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
    }

    pub(super) fn scroll_down(&mut self) {
        self.vertical_scroll = (self.vertical_scroll + 1).min(self.max_scroll);
    }

    pub(super) fn scroll_page_up(&mut self, scroll_amount: u16) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(scroll_amount);
    }

    pub(super) fn scroll_page_down(&mut self, scroll_amount: u16) {
        self.vertical_scroll = (self.vertical_scroll + scroll_amount).min(self.max_scroll);
    }

    pub(super) fn scroll_to_top(&mut self) {
        self.vertical_scroll = 0;
    }

    pub(super) fn scroll_to_bottom(&mut self) {
        self.vertical_scroll = self.max_scroll;
    }

    pub(super) fn render_block(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        config: &Config,
        is_focused: bool,
    ) -> Rect {
        let block = Block::default()
            .borders(Borders::all())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(if is_focused {
                config.theme.border_focused()
            } else {
                config.theme.border()
            });

        // let scroll_thumb_icon = config.scroll_thumb_icon.to_string();
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(config.scrollbar_set())
            .style(config.theme.eff_border(is_focused));

        self.scrollbar_state = self
            .scrollbar_state
            .position(self.vertical_scroll as usize)
            .content_length(self.max_scroll as usize);

        let inner_area = block.inner(area);
        block.render(area, buf);
        StatefulWidget::render(
            scrollbar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            buf,
            &mut self.scrollbar_state,
        );
        inner_area
    }

    pub(super) fn generate_header<'a>(
        &'a self,
        model_data: &'a ArticleContentModelData,
        config: &'a Config,
    ) -> Vec<Line<'a>> {
        let Some(article) = model_data.article() else {
            return vec![];
        };

        let title = html_sanitize(article.title.as_deref().unwrap_or("unknown title"));
        let feed_label: String = if let Some(feed) = model_data.feed() {
            html_sanitize(&feed.label)
        } else {
            article.feed_id.as_str().into()
        };

        let tags = model_data.tags().as_deref().unwrap_or_default();
        let mut tags_and_enclosures = tags
            .iter()
            .flat_map(|tag| {
                let mut line = NewsFlashUtils::tag_to_line(tag, config, None);
                line.spans.push(Span::from(" "));
                line
            })
            .collect::<Vec<Span>>();

        let enclosures = model_data.enclosures().clone().unwrap_or_default();

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            &enclosures,
            |enclosure| enclosure.is_video(),
            config.enclosure_video_icon,
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            &enclosures,
            |enclosure| enclosure.is_image(),
            config.enclosure_image_icon,
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            &enclosures,
            |enclosure| enclosure.is_audio(),
            config.enclosure_audio_icon,
        ));

        let author = html_sanitize(
            article
                .author
                .as_deref()
                .map(|author| format!(" by {author}"))
                .as_deref()
                .unwrap_or(""),
        );

        let date_string: String = article
            .date
            .with_timezone(&chrono::Local)
            .format(&config.date_format)
            .to_string();

        let summary_lines = vec![
            Line::from(vec![
                Span::from(date_string).style(config.theme.header()),
                Span::from("  ").style(config.theme.header()),
                Span::from(feed_label).style(config.theme.header()),
            ]),
            Line::styled(title, config.theme.paragraph()),
            Line::styled(author, config.theme.paragraph()),
            Line::from(tags_and_enclosures),
        ];

        summary_lines
    }

    pub(super) fn render_header(
        &mut self,
        model_data: &ArticleContentModelData,
        config: &Config,
        inner_area: Rect,
        buf: &mut Buffer,
    ) {
        let thumbnail_constraint = if config.thumbnail_show {
            config.thumbnail_width.as_constraint()
        } else {
            Constraint::Length(0)
        };

        let [thumbnail_chunk, header_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(ratatui::layout::Flex::Start)
            .constraints(vec![thumbnail_constraint, Constraint::Min(1)])
            // .margin(1)
            .spacing(1)
            .areas::<2>(inner_area);

        if config.thumbnail_show {
            self.render_thumbnail(model_data, config, thumbnail_chunk, buf);
        }

        let header_lines = self.generate_header(model_data, config);
        let paragraph = Paragraph::new(header_lines).wrap(Wrap { trim: true });
        paragraph.render(header_chunk, buf);
    }

    pub(super) fn render_summary(
        &mut self,
        model_data: &ArticleContentModelData,
        config: &Config,
        inner_area: Rect,
        buf: &mut Buffer,
    ) {
        let Some(article) = model_data.article() else {
            return;
        };

        let [header_chunk, summary_chunk] = Layout::default()
            .direction(Direction::Vertical)
            .flex(ratatui::layout::Flex::Start)
            .constraints([
                config.thumbnail_height.as_constraint(),
                config
                    .thumbnail_height
                    .as_complementary_constraint(inner_area.width.saturating_sub(3)),
            ])
            .horizontal_margin(2)
            .vertical_margin(1)
            .spacing(1)
            .areas::<2>(inner_area);

        self.scrollbar_state = ScrollbarState::default();
        self.render_header(model_data, config, header_chunk, buf);

        let mut summary = article.summary.clone().unwrap_or("".into());
        summary = ArticleContentModelData::clean_string(&summary);
        let summary_paragraph = Paragraph::new(Line::from(
            Span::from(summary).style(config.theme.paragraph()),
        ))
        .wrap(Wrap { trim: true });
        summary_paragraph.render(summary_chunk, buf);
    }

    pub(super) fn render_thumbnail(
        &mut self,
        model_data: &ArticleContentModelData,
        config: &Config,
        thumbnail_chunk: Rect,
        buf: &mut Buffer,
    ) {
        let centered_layout = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center);

        match &mut self.image {
            Some(image) => {
                let mut stateful_image = StatefulImage::new();
                if config.thumbnail_resize {
                    stateful_image =
                        stateful_image.resize(Resize::Scale(Some(FilterType::Lanczos3)));
                }
                let [centered_chunk] = centered_layout
                    .constraints([Constraint::Fill(1)])
                    .areas(thumbnail_chunk);
                stateful_image.render(centered_chunk, buf, image);
            }
            None if *model_data.thumbnail_fetch_running()
                || model_data.thumbnail_fetch_successful().is_none() =>
            {
                let throbber = Throbber::default()
                    .throbber_style(config.theme.header())
                    .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
                    .use_type(WhichUse::Spin);
                let [centered_chunk] = centered_layout
                    .constraints([Constraint::Length(1)])
                    .areas(thumbnail_chunk);
                StatefulWidget::render(
                    throbber,
                    centered_chunk,
                    buf,
                    &mut self.thumbnail_fetching_throbber,
                );
            }
            _ => {
                let mut stateful_image = StatefulImage::new();
                if config.thumbnail_resize {
                    stateful_image = stateful_image.resize(Resize::Fit(Some(FilterType::Lanczos3)))
                }
                let [centered_chunk] = centered_layout
                    .constraints([Constraint::Fill(1)])
                    .areas(thumbnail_chunk);
                stateful_image.render(centered_chunk, buf, &mut self.placeholder_image);
            }
        }
    }

    pub(super) fn render_fat_article(
        &mut self,
        model_data: &ArticleContentModelData,
        distraction_free: bool,
        config: &Config,
        inner_area: Rect,
        buf: &mut Buffer,
    ) {
        let show_header = !distraction_free || config.zen_mode_show_header;

        let [summary_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Start)
            .constraints([
                Constraint::Length(if show_header { 5 } else { 0 }),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .vertical_margin(1)
            .spacing(1)
            .areas::<2>(inner_area);

        let text_constraint = if distraction_free {
            Constraint::Max(config.text_max_width)
        } else {
            Constraint::Percentage(100)
        };

        if show_header {
            let [header_area] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(ratatui::layout::Flex::Center)
                .constraints([text_constraint])
                .areas(summary_area);
            self.render_header(model_data, config, header_area, buf);
        }

        let [paragraph_area] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(ratatui::layout::Flex::Center)
            .constraints([text_constraint])
            .areas(content_area);

        let Some(fat_article) = model_data.fat_article() else {
            return;
        };

        let text: Text<'_> = if config.content_preferred_type == ArticleContentType::Markdown
            && let Some(html) = fat_article.scraped_content.as_deref()
        {
            // Use the cached markdown content from model
            if let Some(markdown) = model_data.markdown_content() {
                info!("markdown available");
                tui_markdown::from_str(markdown)
            } else {
                info!("no markdown available, falling back to html2text");
                // Fallback - convert to plain text instead of markdown to avoid lifetime issues
                let plain_text = news_flash::util::html2text::html2text(html);
                Text::from(plain_text)
            }
        } else if let Some(plain_text) = fat_article.plain_text.as_deref() {
            info!("rendering plain text content");
            Text::from(plain_text)
        } else {
            info!("no content available");
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

fn to_enclosure_bubble<P>(
    config: &Config,
    enclosures: &'_ [Enclosure],
    predicate: P,
    icon: char,
) -> Vec<Span<'static>>
where
    P: FnMut(&Enclosure) -> bool,
{
    let any_enclosures = enclosures.iter().any(predicate);
    if any_enclosures {
        to_bubble(Span::styled(format!("{}", icon), config.theme.paragraph())).spans
    } else {
        Default::default()
    }
}

impl Widget for &mut ArticleContent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let inner_area = self
            .view_data
            .render_block(area, buf, &self.config, self.is_focused);

        if !self.model_data.article().is_some() {
            return;
        }

        if self.model_data.fat_article().is_some() {
            self.view_data.render_fat_article(
                &self.model_data,
                self.is_distraction_free,
                &self.config,
                inner_area,
                buf,
            );
        } else if self.model_data.article().is_some() {
            self.view_data
                .render_summary(&self.model_data, &self.config, inner_area, buf);
        }
    }
}
