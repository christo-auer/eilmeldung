pub mod prelude {
    pub use super::html_sanitize;
}

pub fn html_sanitize(html_escaped_string: &str) -> String {
    htmlescape::decode_html(&html_escaped_string.replace("＆", "&"))
        .unwrap_or(html_escaped_string.to_owned())
}
