use super::model::ArticleContentModelData;
use crate::{
    prelude::*,
    ui::article_content::{
        markdown_to_text::HintUrlMap,
        state::{ArticleBody, LoadedArticle, ThumbnailState},
    },
};

use std::{io::Cursor, sync::Arc};

use getset::{Getters, MutGetters};
use image::{DynamicImage, ImageReader};
use limner::{RenderResult, render_image::ImageStore};
use news_flash::models::{ArticleID, Enclosure, Thumbnail};
use ratatui::layout::Flex;
use ratatui_image::{
    FilterType, Resize, ResizeEncodeRender, picker::Picker, protocol::StatefulProtocol,
};

use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse};

const NO_THUMB_PLACEHOLDER: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/no-thumb.png"));

#[derive(Clone, Hash, Eq, PartialEq)]
pub(super) enum HintTarget {
    Image(String),
    Web(String),
}

impl HintTarget {
    pub(super) fn url(&self) -> &str {
        match self {
            HintTarget::Image(url) => url,
            HintTarget::Web(url) => url,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct RenderKey {
    article_id: ArticleID,
    width: u16,
    content: String,
}

struct RenderCache {
    render_key: RenderKey,
    thumbnail_protocol: Option<StatefulProtocol>,
    render_result: RenderResult,
    image_store: ImageStore,
    hint_url_map: HintUrlMap,
}

#[derive(Getters, MutGetters)]
pub struct ArticleContentViewData {
    // Scroll state
    vertical_scroll: u16,
    max_scroll: u16,

    scrollbar_state: ScrollbarState,

    // Image rendering state
    placeholder_image: StatefulProtocol,

    // picker
    picker: Option<Picker>,
    fallback_picker: Picker,

    // renderer cache
    render_cache: Option<RenderCache>,

    load_visible_images: bool,
    load_image_hint: Option<String>,

    // Throbber state for loading animations
    thumbnail_fetching_throbber: ThrobberState,

    // needed to fetch article images
    news_flash_utils: Arc<NewsFlashUtils>,
}

impl ArticleContentViewData {
    pub fn new(news_flash_utils: Arc<NewsFlashUtils>, config: &Config) -> Self {
        Self {
            vertical_scroll: 0,
            max_scroll: 0,
            placeholder_image: Self::prepare_placeholder_image(&Picker::halfblocks(), config),
            picker: None,
            fallback_picker: Picker::halfblocks(),
            thumbnail_fetching_throbber: ThrobberState::default(),
            scrollbar_state: ScrollbarState::default(),
            render_cache: None,
            load_visible_images: false,
            load_image_hint: None,
            news_flash_utils,
        }
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

    pub(super) fn on_image_fetch_finished(&mut self, url: &str, image: DynamicImage) -> ModelState {
        if let Some(RenderCache { image_store, .. }) = self.render_cache.as_mut() {
            image_store.finish(url, Some(image));
            ModelState::Dirty
        } else {
            log::debug!("got an image but don't have an article to render");
            ModelState::Clean
        }
    }

    pub(super) fn on_image_fetch_failed(&mut self, url: &str) -> ModelState {
        if let Some(RenderCache { image_store, .. }) = self.render_cache.as_mut() {
            image_store.finish(url, None);
            ModelState::Dirty
        } else {
            log::debug!("got a failed image but don't have an article to render");
            ModelState::Clean
        }
    }

    pub(super) fn force_update_render_cache(&mut self) -> ModelState {
        self.render_cache = None;
        ModelState::Dirty
    }

    fn update_render_cache(
        &mut self,
        model_data: &ArticleContentModelData,
        width: u16,
        config: &Config,
    ) -> ModelState {
        let new_render_key = model_data
            .loaded_article()
            .as_ref()
            .map(|loaded_article| RenderKey {
                article_id: loaded_article.article_id().to_owned(),
                width,
                content: loaded_article.content().to_owned(),
            });

        if self
            .render_cache
            .as_ref()
            .map(|RenderCache { render_key, .. }| render_key)
            == new_render_key.as_ref()
        {
            log::debug!("render key has not changed: {new_render_key:?}");
            return ModelState::Clean;
        }

        let Some(render_key) = new_render_key else {
            self.render_cache = None;
            return ModelState::Dirty;
        };

        let (render_result, image_store, hint_url_map) = Self::generate_content_render_cache(
            self.picker().to_owned(),
            &render_key.content,
            width,
            config,
        );

        self.render_cache = Some(RenderCache {
            render_key,
            render_result,
            hint_url_map,
            image_store,
            thumbnail_protocol: None,
        });

        ModelState::Dirty
    }

    fn render_block(
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

    fn generate_header<'a>(
        &'a self,
        model_data: &'a ArticleContentModelData,
        config: &'a Config,
    ) -> Vec<Line<'a>> {
        let Some(loaded_article) = model_data.loaded_article().as_ref() else {
            return vec![];
        };

        let title = html_sanitize(
            loaded_article
                .article
                .title
                .as_deref()
                .unwrap_or("unknown title"),
        );
        let feed_label = &loaded_article.feed().label;

        let mut tags_and_enclosures = loaded_article
            .tags()
            .iter()
            .flat_map(|tag| {
                let mut line = NewsFlashUtils::tag_to_line(tag, config, None);
                line.spans.push(Span::from(" "));
                line
            })
            .collect::<Vec<Span>>();

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            loaded_article.enclosures(),
            |enclosure| enclosure.is_video(),
            config.icon_set.enclosure_video_icon(),
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            loaded_article.enclosures(),
            |enclosure| enclosure.is_image(),
            config.icon_set.enclosure_image_icon(),
        ));

        tags_and_enclosures.append(&mut to_enclosure_bubble(
            config,
            loaded_article.enclosures(),
            |enclosure| enclosure.is_audio(),
            config.icon_set.enclosure_audio_icon(),
        ));

        let author = html_sanitize(
            loaded_article
                .article()
                .author
                .as_deref()
                .map(|author| format!(" by {author}"))
                .as_deref()
                .unwrap_or(""),
        );

        let date_string: String = loaded_article
            .article()
            .date
            .with_timezone(&chrono::Local)
            .format(&config.date_format)
            .to_string();

        let mut title_line = Line::from(vec![
            Span::from(date_string).style(config.theme.header()),
            Span::from("  ").style(config.theme.header()),
            Span::from(feed_label).style(config.theme.header()),
        ]);

        if matches!(loaded_article.body(), ArticleBody::Piped { .. }) {
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

    pub fn article_layout(
        area: Rect,
        distraction_free: bool,
        config: &Config,
    ) -> (Option<Rect>, Rect) {
        let show_header = !distraction_free || config.zen_mode_show_header;

        let text_constraint = if distraction_free {
            Constraint::Max(config.text_max_width)
        } else {
            Constraint::Percentage(100)
        };

        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Start)
            .constraints([
                Constraint::Length(if show_header { 5 } else { 0 }),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .vertical_margin(1)
            .spacing(if show_header { 1 } else { 0 })
            .areas::<2>(area);

        let text_area = |available_area: Rect| {
            Layout::default()
                .direction(Direction::Horizontal)
                .flex(ratatui::layout::Flex::Center)
                .constraints([text_constraint])
                .areas::<1>(available_area)[0]
        };

        (
            show_header.then(|| text_area(header_area)),
            text_area(content_area),
        )
    }

    pub(super) fn render_thumbnail(
        &mut self,
        model_data: &ArticleContentModelData,
        config: &Config,
        thumbnail_chunk: Rect,
        buf: &mut Buffer,
    ) {
        let picker = self.picker().to_owned();
        let Some(render_cache) = self.render_cache.as_mut() else {
            return;
        };

        let Some(LoadedArticle {
            thumbnail_state, ..
        }) = model_data.loaded_article().as_ref()
        else {
            return;
        };

        let [centered_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Fill(1)])
            .areas::<1>(thumbnail_chunk);

        match thumbnail_state {
            ThumbnailState::Failed => {
                self.placeholder_image.resize_encode_render(
                    &Resize::Fit(Some(FilterType::Lanczos3)),
                    centered_chunk,
                    buf,
                );
            }
            ThumbnailState::None | ThumbnailState::Fetching => {
                let [_, throbber_chunk, _] = Layout::horizontal([
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .vertical_margin(1)
                .areas::<3>(centered_chunk);
                let throbber = Throbber::default()
                    .throbber_style(config.theme.header())
                    .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
                    .use_type(WhichUse::Spin);
                StatefulWidget::render(
                    throbber,
                    throbber_chunk,
                    buf,
                    &mut self.thumbnail_fetching_throbber,
                );
            }
            ThumbnailState::Fetched(thumbnail) => {
                if render_cache.thumbnail_protocol.is_none() {
                    let mut thumbnail = Self::load_thumbnail(&picker, thumbnail, config);
                    thumbnail = Some(
                        thumbnail
                            .unwrap_or_else(|| Self::prepare_placeholder_image(&picker, config)),
                    );
                    render_cache.thumbnail_protocol = thumbnail;
                }

                if let Some(thumbnail_protocol) = render_cache.thumbnail_protocol.as_mut() {
                    thumbnail_protocol.resize_encode_render(
                        &Resize::Fit(Some(FilterType::Lanczos3)),
                        centered_chunk,
                        buf,
                    );
                }
            }
        }
    }

    fn load_thumbnail(
        picker: &Picker,
        thumbnail: &Thumbnail,
        config: &Config,
    ) -> Option<StatefulProtocol> {
        let data = thumbnail.data.as_ref()?;

        let cursor = Cursor::new(data);

        let mut image = ImageReader::new(cursor)
            .with_guessed_format()
            .ok()
            .and_then(|image_reader| image_reader.decode().ok())?;
        image = apply_image_enhancements(image, picker, config);
        Some(picker.new_resize_protocol(image))
    }

    pub(super) fn render_article(
        &mut self,
        model_data: &ArticleContentModelData,
        distraction_free: bool,
        config: &Config,
        inner_area: Rect,
        buf: &mut Buffer,
    ) {
        let vertical_scroll = self.vertical_scroll;
        let picker = self.picker().to_owned();

        let (header_area, paragraph_area) =
            Self::article_layout(inner_area, distraction_free, config);

        let markdown_rendering_width = paragraph_area.width;

        let model_state = self.update_render_cache(model_data, markdown_rendering_width, config);

        // reset scroll if render cache has updated
        if model_state.is_dirty() {
            self.vertical_scroll = 0;
        }

        if let Some(header_area) = header_area {
            self.render_header(model_data, config, header_area, buf);
        }

        let Some(render_cache) = self.render_cache.as_mut() else {
            log::trace!("no render cache");
            return;
        };

        let mut lines = render_cache.render_result.lines.to_owned();

        let image_rows = config
            .article_images_height
            .into_length(inner_area.height)
            .raw_value();

        let render_positions = render_cache.image_store.prepare(
            &mut lines,
            &render_cache.render_result.images,
            markdown_rendering_width,
            image_rows,
        );

        log::trace!("rendering article");
        self.max_scroll =
            (render_cache.image_store.total_rows() as u16).saturating_sub(paragraph_area.height);
        self.vertical_scroll = vertical_scroll.min(self.max_scroll);
        let image_max_width = u32::from(picker.font_size().width * paragraph_area.width);
        let image_rows = u32::from(picker.font_size().height * image_rows);
        if self.load_visible_images || config.article_images_auto_load {
            // find visible or nearly visible images which are not yet downloaded
            for url in render_cache.image_store.start_visible(
                self.vertical_scroll,
                paragraph_area.height,
                if config.article_images_auto_load {
                    paragraph_area.height
                } else {
                    0
                },
            ) {
                self.news_flash_utils.fetch_image_with_cache(
                    &url,
                    image_max_width,
                    image_rows,
                    to_corner_radius(config, &picker),
                    config.theme.color_palette().image_drop_shadow,
                    to_drop_shadow_width(config, &picker),
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
                to_corner_radius(config, &picker),
                config.theme.color_palette().image_drop_shadow,
                to_drop_shadow_width(config, &picker),
            );
        }

        let content = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.vertical_scroll, 0));

        content.render(paragraph_area, buf);

        render_cache.image_store.render(
            buf,
            paragraph_area,
            &render_positions,
            self.vertical_scroll,
        );
    }

    pub(super) fn resolve_hint(&self, hint: &str) -> Option<&HintTarget> {
        self.render_cache
            .as_ref()
            .and_then(|render_cache| render_cache.hint_url_map.get(hint))
    }

    pub(super) fn picker(&self) -> &Picker {
        self.picker.as_ref().unwrap_or(&self.fallback_picker)
    }

    pub fn picker_updated(&mut self, picker: &Picker, config: &Config) -> ModelState {
        self.picker = Some(picker.to_owned());

        if let Some(RenderCache {
            image_store,
            thumbnail_protocol,
            ..
        }) = self.render_cache.as_mut()
        {
            image_store.clear();
            *thumbnail_protocol = None;
        };

        self.placeholder_image = Self::prepare_placeholder_image(picker, config);
        ModelState::Dirty
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

    pub(super) fn load_image(&mut self, url: &str) {
        self.load_image_hint = Some(url.to_owned());
    }

    pub(super) fn load_visible_images(&mut self) {
        self.load_visible_images = true;
    }

    pub(super) fn tick(&mut self, model_data: &ArticleContentModelData) -> ModelState {
        if model_data
            .loaded_article()
            .as_ref()
            .map(|loaded_article| {
                matches!(loaded_article.thumbnail_state, ThumbnailState::Fetching)
            })
            .unwrap_or(false)
        {
            self.thumbnail_fetching_throbber.calc_next();
            ModelState::Dirty
        } else {
            ModelState::Clean
        }
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

        self.view_data.render_article(
            &self.model_data,
            self.is_distraction_free,
            &self.config,
            inner_area,
            buf,
        );
    }
}
