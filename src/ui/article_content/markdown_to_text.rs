use std::{collections::HashMap, sync::Arc};

use crate::{prelude::*, ui::article_content::view::ArticleContentViewData};
use limner::{
    MarkdownStyle, RenderResult, SpanFormatter, render_image::ImageStore, render_markdown,
};
use std::sync::Mutex;

impl ArticleContentViewData {
    #[allow(clippy::too_many_arguments)]
    fn format_link(
        icon: char,
        hint: &str,
        alt: &str,
        url: &str,
        hint_style: Style,
        alt_style: Style,
        url_style: Style,
        show_url: bool,
    ) -> Vec<Span<'static>> {
        let mut spans = vec![
            Span::styled(format!("{hint}{icon}"), hint_style),
            Span::styled(alt.to_owned(), alt_style),
        ];
        if show_url {
            spans.push(Span::styled(format!("({})", url), url_style));
        }
        spans
    }

    pub(super) fn markdown_to_text(&mut self, markdown: &str, config: &Config) -> RenderResult {
        let url_for_hint = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        let iterator = config.hint_type.iter();
        let hint_iterator = Arc::new(Mutex::new(iterator));
        let show_url = config.content_show_urls;

        let inner_link_url_for_hint = url_for_hint.clone();
        let inner_link_hint_iterator = hint_iterator.clone();

        let link_alt_text_style = config.theme.link_text_style();
        let link_url_text_style = config.theme.link_url_style();
        let link_hint_style = config.theme.link_hint_style();

        let inner_image_url_for_hint = url_for_hint.clone();
        let inner_image_hint_iterator = hint_iterator.clone();

        let image_alt_text_style = config.theme.link_text_style();
        let image_url_text_style = config.theme.link_url_style();
        let image_hint_style = config.theme.link_hint_style();

        let image_icon = config.icon_set.image_icon();
        let url_icon = config.icon_set.url_icon();

        let link_formatter = SpanFormatter::new(move |element| {
            let mut url_for_hint = inner_link_url_for_hint.lock().unwrap(); // unwrap is save here: locking with sync calls
            let hint = inner_link_hint_iterator.lock().unwrap().next().unwrap(); // unwrap is save here: locking with sync calls
            url_for_hint
                .entry(hint.to_owned())
                .or_insert(element.url.to_owned());

            Self::format_link(
                url_icon,
                &hint,
                element.text,
                element.url,
                link_hint_style,
                link_alt_text_style,
                link_url_text_style,
                show_url,
            )
        });

        let image_formatter = SpanFormatter::new(move |element| {
            let mut url_for_hint = inner_image_url_for_hint.lock().unwrap();
            let hint = inner_image_hint_iterator.lock().unwrap().next().unwrap(); // unwrap is save here: locking with sync calls
            url_for_hint
                .entry(hint.to_owned())
                .or_insert(element.url.to_owned());
            Self::format_link(
                image_icon,
                &hint,
                element.text,
                element.url,
                image_hint_style,
                image_alt_text_style,
                image_url_text_style,
                show_url,
            )
        });

        let style = MarkdownStyle {
            link_formatter: Some(link_formatter),
            image_formatter: Some(image_formatter),
            link_prefix: "",
            image_prefix: "",
            paragraph: config.theme.paragraph(),
            heading_1: config.theme.header(),
            heading_2: config.theme.header(),
            heading_3: config.theme.header(),
            code_block: config.theme.paragraph(),
            code_block_border_style: config.theme.paragraph(),

            ..Default::default()
        };

        let rendered = render_markdown(markdown, &style, config.text_max_width);

        self.url_for_hint = url_for_hint.lock().unwrap().to_owned();
        self.create_image_store(config);

        rendered
    }

    pub(super) fn create_image_store(
        &mut self,
        config: &Config,
        // url_for_hint: &HashMap<String, String>,
    ) {
        let loading_icon = config.icon_set.loading_icon();
        let failed_icon = config.icon_set.image_failed_icon();
        let hint_for_url_loading = Arc::new(
            self.url_for_hint
                .iter()
                .map(|(hint, url)| (url.to_owned(), hint.to_owned()))
                .collect::<HashMap<String, String>>(),
        );
        let hint_for_url_failed = Arc::clone(&hint_for_url_loading);
        let alt_style = config.theme.link_text_style();
        let url_style = config.theme.link_url_style();
        let hint_style = config.theme.link_hint_style();
        let show_url = config.content_show_urls;

        self.image_store = Some(
            ImageStore::new(self.picker.clone())
                .with_loading_line(move |pending_image| {
                    let hint = hint_for_url_loading
                        .get(&pending_image.info.url)
                        .map(String::as_str)
                        .unwrap_or_default();
                    Self::format_link(
                        loading_icon,
                        hint,
                        &pending_image.info.alt,
                        &pending_image.info.url,
                        hint_style,
                        alt_style,
                        url_style,
                        show_url,
                    )
                    .into()
                })
                .with_failed_line(move |pending_image| {
                    let hint = hint_for_url_failed
                        .get(&pending_image.info.url)
                        .map(String::as_str)
                        .unwrap_or_default();
                    Self::format_link(
                        failed_icon,
                        hint,
                        &pending_image.info.alt,
                        &pending_image.info.url,
                        hint_style,
                        alt_style,
                        url_style,
                        show_url,
                    )
                    .into()
                })
                .keep_placeholders(),
        );
    }
}
