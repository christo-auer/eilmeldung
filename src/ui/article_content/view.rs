use super::model::ArticleContentModelData;
use crate::prelude::*;

use std::{collections::HashMap, io::Cursor, sync::Arc};

use getset::{Getters, MutGetters};
use image::{DynamicImage, ImageReader};
use limner::{RenderResult, render_image::ImageStore};
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
    pub(super) picker: Picker,

    // article images
    pub(super) markdown_rendering: Option<RenderResult>,
    pub(super) markdown_rendering_width: Option<u16>,
    pub(super) image_store: Option<ImageStore>,
    load_visible_images: bool,
    load_image_hint: Option<String>,

    // Throbber state for loading animations
    thumbnail_fetching_throbber: ThrobberState,

    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    pub(super) url_for_hint: HashMap<String, String>,
    // needed to fetch article images
    news_flash_utils: Arc<NewsFlashUtils>,
}

impl ArticleContentViewData {
    pub fn new(news_flash_utils: Arc<NewsFlashUtils>, config: &Config) -> Self {
        let picker = Picker::from_query_stdio().unwrap();
        Self {
            vertical_scroll: 0,
            max_scroll: 0,
            image: None,
            placeholder_image: Self::prepare_placeholder_image(&picker, config),
            picker: picker.clone(),
            thumbnail_fetching_throbber: ThrobberState::default(),
            scrollbar_state: ScrollbarState::default(),
            url_for_hint: Default::default(),
            image_store: None,
            load_visible_images: false,
            load_image_hint: None,
            markdown_rendering: None,
            markdown_rendering_width: None,
            news_flash_utils,
        }
    }

    pub(super) fn update(&mut self, model_data: &ArticleContentModelData, _config: Arc<Config>) {
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

    pub(super) fn clear(&mut self, config: &Config) {
        self.image = None;
        if let Some(image_store) = self.image_store.as_mut() {
            image_store.clear();
        }
        self.placeholder_image = Self::prepare_placeholder_image(&self.picker, config);
        self.vertical_scroll = 0;
        self.markdown_rendering = None;
        self.markdown_rendering_width = None;
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

    pub(super) fn tick_throbbers(&mut self) {
        self.thumbnail_fetching_throbber.calc_next();
    }

    pub(super) fn scroll_up(&mut self, amount: u16) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(amount);
    }

    pub(super) fn scroll_down(&mut self, amount: u16) {
        self.vertical_scroll = (self.vertical_scroll + amount).min(self.max_scroll);
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

    pub(super) fn on_image_fetch_finished(&mut self, url: &str, image: DynamicImage) {
        if let Some(image_store) = self.image_store.as_mut() {
            image_store.finish(url, Some(image));
        }
    }

    pub(super) fn on_image_fetch_failed(&mut self, url: &str) {
        if let Some(image_store) = self.image_store.as_mut() {
            image_store.finish(url, None);
        }
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
            .border_type(config.border_theme.eff_type(is_focused))
            .merge_borders(config.border_theme.framing.eff_merge_strategy())
            .border_style(if is_focused {
                config.theme.border_focused()
            } else {
                config.theme.border()
            })
            .title_bottom(
                if self.max_scroll > 0 && config.content_show_position {
                    Line::styled(
                        format!(
                            " {}% ",
                            ((self.vertical_scroll as f64 / self.max_scroll as f64) * 100.0) as u16
                        ),
                        config.theme.header(),
                    )
                } else {
                    "".into()
                }
                .alignment(HorizontalAlignment::Right),
            );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(config.border_theme.scrollbar_set(is_focused))
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
            config.icon_set.enclosure_video_icon(),
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            &enclosures,
            |enclosure| enclosure.is_image(),
            config.icon_set.enclosure_image_icon(),
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            &enclosures,
            |enclosure| enclosure.is_audio(),
            config.icon_set.enclosure_audio_icon(),
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

        let mut title_line = Line::from(vec![
            Span::from(date_string).style(config.theme.header()),
            Span::from("  ").style(config.theme.header()),
            Span::from(feed_label).style(config.theme.header()),
        ]);

        if model_data.filtered_markdown_content().is_some() {
            title_line.spans.push(" ".into());
            title_line.spans.append(
                &mut to_bubble(
                    Span::styled(
                        config.icon_set.piped_icon().to_string(),
                        config.theme.highlighted(&Default::default()),
                    ),
                    config,
                )
                .spans,
            );
        }

        let summary_lines = vec![
            title_line,
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

        let vertical_scroll = self.vertical_scroll;

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

        // prefer filtered content
        let markdown = model_data
            .filtered_markdown_content()
            .as_deref()
            .or(model_data.markdown_content().as_deref())
            .unwrap_or_default();

        let mut markdown_rendering_width = self
            .markdown_rendering_width
            .take()
            .unwrap_or(paragraph_area.width);

        if markdown_rendering_width != paragraph_area.width {
            self.markdown_rendering.take();
            self.markdown_rendering_width.replace(paragraph_area.width);
            markdown_rendering_width = paragraph_area.width;
            if let Some(image_store) = self.image_store.as_mut() {
                image_store.clear();
            }
        }

        let markdown_rendering = self
            .markdown_rendering
            .take()
            .unwrap_or_else(|| self.markdown_to_text(markdown, markdown_rendering_width, config));

        let mut lines = markdown_rendering.lines.clone();

        let image_rows = config
            .article_image_height
            .into_length(inner_area.height)
            .raw_value();

        let Some(image_store) = self.image_store.as_mut() else {
            unreachable!();
        };

        let render_positions = image_store.prepare(
            &mut lines,
            &markdown_rendering.images,
            markdown_rendering_width,
            image_rows,
        );

        self.max_scroll = (image_store.total_rows() as u16).saturating_sub(paragraph_area.height);
        self.vertical_scroll = vertical_scroll.min(self.max_scroll);
        let image_max_width = u32::from(self.picker.font_size().width * paragraph_area.width);
        let image_rows = u32::from(self.picker.font_size().height * image_rows);
        if self.load_visible_images || config.article_image_auto_load {
            // find visible or nearly visible images which are not yet downloaded
            for url in image_store.start_visible(
                self.vertical_scroll,
                paragraph_area.height,
                if config.article_image_auto_load {
                    paragraph_area.height
                } else {
                    0
                },
            ) {
                self.news_flash_utils.fetch_image_with_cache(
                    &url,
                    image_max_width,
                    image_rows,
                    to_corner_radius(config, &self.picker),
                    config.theme.color_palette().image_drop_shadow,
                    to_drop_shadow_width(config, &self.picker),
                );
            }
            self.load_visible_images = false;
        }

        // load image if requested via hint
        if let Some(url) = self.load_image_hint.take() {
            self.news_flash_utils.fetch_image_with_cache(
                &url,
                image_max_width,
                image_rows,
                to_corner_radius(config, &self.picker),
                config.theme.color_palette().image_drop_shadow,
                to_drop_shadow_width(config, &self.picker),
            );
        }

        let content = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.vertical_scroll, 0));

        content.render(paragraph_area, buf);

        image_store.render(buf, paragraph_area, &render_positions, self.vertical_scroll);

        self.markdown_rendering_width
            .replace(markdown_rendering_width);
        self.markdown_rendering.replace(markdown_rendering);
    }

    pub fn picker_updated(&mut self, picker: &Picker, config: &Config) -> color_eyre::Result<()> {
        self.picker = picker.to_owned();
        self.create_image_store(config);
        self.placeholder_image = Self::prepare_placeholder_image(picker, config);

        Ok(())
    }

    fn prepare_placeholder_image(picker: &Picker, config: &Config) -> StatefulProtocol {
        let cursor = Cursor::new(NO_THUMB_PLACEHOLDER);
        let mut image = ImageReader::new(cursor)
            .with_guessed_format()
            .unwrap() // OK as content is checked
            .decode()
            .unwrap(); // OK as content is checked

        image = add_drop_shadow(
            &image,
            config.theme.color_palette().image_drop_shadow,
            0,
            to_corner_radius(config, picker),
        );

        picker.new_resize_protocol(image)
    }

    pub(crate) fn load_image(&mut self, url: &str) {
        self.load_image_hint = Some(url.to_owned());
    }

    pub(crate) fn load_visible_images(&mut self) {
        self.load_visible_images = true;
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
        to_bubble(
            Span::styled(format!("{}", icon), config.theme.paragraph()),
            config,
        )
        .spans
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

        if self.model_data.fat_article().is_some()
            || self.model_data.filtered_markdown_content().is_some()
        {
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
