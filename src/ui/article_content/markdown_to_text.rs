use std::{collections::HashMap, sync::Arc};

use crate::{prelude::*, ui::article_content::view::ArticleContentViewData};
use limner::{MarkdownStyle, RenderResult, SpanFormatter, render_markdown};
use std::sync::Mutex;

impl ArticleContentViewData {
    pub(super) fn markdown_to_text(&mut self, markdown: &str, config: &Config) -> RenderResult {
        log::trace!("M2T");

        let url_for_hint = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        let iterator = config.hint_type.iter();
        let hint_iterator = Arc::new(Mutex::new(iterator));
        let show_url = config.content_show_urls;

        let inner_link_url_for_hint = url_for_hint.clone();
        let inner_link_hint_iterator = hint_iterator.clone();
        let link_alt_text_style = Style::new()
            .fg(*config.theme.color_palette().accent_primary())
            .add_modifier(Modifier::UNDERLINED);
        let link_url_text_style = Style::new().fg(*config.theme.color_palette().foreground());
        let link_hint_style = Style::new()
            .fg(*config.theme.color_palette().highlight())
            .add_modifier(Modifier::BOLD);

        let inner_image_url_for_hint = url_for_hint.clone();
        let inner_image_hint_iterator = hint_iterator.clone();
        let image_alt_text_style = Style::new()
            .fg(*config.theme.color_palette().accent_primary())
            .add_modifier(Modifier::UNDERLINED);
        let image_url_text_style = Style::new().fg(*config.theme.color_palette().foreground());
        let image_hint_style = link_hint_style;

        let image_icon = config.icon_set.image_icon();
        let url_icon = config.icon_set.url_icon();

        let link_formatter = SpanFormatter::new(move |element| {
            let mut url_for_hint = inner_link_url_for_hint.lock().unwrap(); // unwrap is save here: locking with sync calls
            let hint = inner_link_hint_iterator.lock().unwrap().next().unwrap(); // unwrap is save here: locking with sync calls

            url_for_hint
                .entry(hint.to_owned())
                .or_insert(element.url.to_owned());
            let mut spans = vec![
                Span::styled(format!("{hint}{url_icon}"), link_hint_style),
                Span::styled(element.text.to_owned(), link_alt_text_style),
            ];
            if show_url {
                spans.push(Span::styled(
                    format!("({})", element.url),
                    link_url_text_style,
                ));
            }

            spans
        });

        let image_formatter = SpanFormatter::new(move |element| {
            let mut url_for_hint = inner_image_url_for_hint.lock().unwrap();
            let hint = inner_image_hint_iterator.lock().unwrap().next().unwrap(); // unwrap is save here: locking with sync calls
            url_for_hint
                .entry(hint.to_owned())
                .or_insert(element.url.to_owned());
            let mut spans = vec![
                Span::styled(format!("{hint}{image_icon}"), image_hint_style),
                Span::styled(element.text.to_owned(), image_alt_text_style),
            ];

            if show_url {
                spans.push(Span::styled(
                    format!("({})", element.url),
                    image_url_text_style,
                ));
            }

            spans
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

        *self.url_for_hint_mut() = url_for_hint.lock().unwrap().to_owned();

        rendered
    }
}
