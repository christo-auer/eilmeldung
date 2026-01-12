use ratatui::{
    style::Modifier,
    text::{Line, Span},
};

pub mod prelude {
    pub use super::html_sanitize;
    pub use super::to_bubble;
}

pub fn html_sanitize(html_escaped_string: &str) -> String {
    htmlescape::decode_html(&html_escaped_string.replace("＆", "&"))
        .unwrap_or(html_escaped_string.to_owned())
}

pub fn to_bubble<'a>(span: Span<'a>) -> Line<'a> {
    let style = span.style;

    Line::from(vec![
        Span::styled("", style), 
        span.style(style.add_modifier(Modifier::REVERSED)),
        Span::styled("", style),
    ])
}
