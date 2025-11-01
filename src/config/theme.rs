use ratatui::style::{Color, Modifier, Style, Stylize};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Theme {
    pub feeds_list_focus_width_percent: u16,
    pub articles_list_focused_width_percent: u16,
    pub articles_list_height_lines: u16,

    pub category: Style,
    pub feed: Style,
    pub header: Style,
    pub paragraph: Style,
    pub tag: Style,
    pub query: Style,
    pub article: Style,
    pub article_highlighted: Style,
    pub statusbar: Style,
    pub tooltip_info: Style,
    pub tooltip_warning: Style,
    pub tooltip_error: Style,
    pub command_line: Style,
    pub inactive: Style,

    pub border_style: Style,
    pub focused_border_style: Style,

    pub unread_modifier: Modifier,
}

impl Default for Theme {
    fn default() -> Self {
        use Color::*;
        Self {
            feeds_list_focus_width_percent: 33,
            articles_list_focused_width_percent: 67,
            articles_list_height_lines: 6,

            category: Style::default().fg(Blue),
            feed: Style::default().fg(White),
            header: Style::default().fg(Magenta),
            paragraph: Style::default().fg(Gray),
            article: Style::default().fg(Gray),
            article_highlighted: Style::default().fg(Yellow).bold(),
            tag: Style::default().fg(Blue),
            query: Style::default().fg(Yellow),
            statusbar: Style::default().fg(Black).bg(Magenta).bold(),
            tooltip_info: Style::default().fg(Black).bg(Magenta),
            tooltip_warning: Style::default().fg(Black).bg(Yellow).bold(),
            tooltip_error: Style::default().fg(Black).bg(Red).bold(),
            command_line: Style::default().bg(DarkGray).fg(Magenta),
            inactive: Style::default().fg(DarkGray),

            border_style: Style::default().fg(Magenta),
            focused_border_style: Style::default().fg(Gray),

            unread_modifier: Modifier::BOLD,
        }
    }
}
