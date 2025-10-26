use ratatui::style::{Color, Modifier, Style, Stylize};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Theme {
    pub feeds_focused_width: u16,
    pub articles_focused_width: u16,
    pub article_content_focused_height: u16,
    pub category: Style,
    pub feed: Style,
    pub header: Style,
    pub paragraph: Style,
    pub tag: Style,
    pub query: Style,
    pub article: Style,
    pub statusbar: Style,
    pub tooltip_info: Style,
    pub tooltip_warning: Style,
    pub tooltip_error: Style,

    pub border_style: Style,
    pub focused_border_style: Style,

    pub unread_modifier: Modifier,
}

impl Default for Theme {
    fn default() -> Self {
        use Color::*;
        Self {
            feeds_focused_width: 33,
            articles_focused_width: 67,
            article_content_focused_height: 66,

            category: Style::default().fg(Blue),
            feed: Style::default().fg(White),
            header: Style::default().fg(Magenta),
            paragraph: Style::default().fg(Gray),
            article: Style::default().fg(Gray),
            tag: Style::default().fg(Blue),
            query: Style::default().fg(Yellow),
            statusbar: Style::default().fg(Black).bg(Magenta).bold(),
            tooltip_info: Style::default().fg(Black).bg(Magenta),
            tooltip_warning: Style::default().fg(Black).bg(Yellow).bold(),
            tooltip_error: Style::default().fg(Black).bg(Red).bold(),

            border_style: Style::default().fg(Magenta),
            focused_border_style: Style::default().fg(Yellow).bold(),

            unread_modifier: Modifier::BOLD,
        }
    }
}
