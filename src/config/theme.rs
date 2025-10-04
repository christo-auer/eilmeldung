use ratatui::style::{Color, Modifier, Style, Stylize};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Theme {
    pub feeds_width: u16,
    pub category: Style,
    pub feed: Style,
    pub header: Style,
    pub tag: Style,
    pub query: Style,

    pub unread_modifier: Modifier,
}

impl Default for Theme {
    fn default() -> Self {
        use Color::*;
        Self {
            feeds_width: 25,

            category: Style::default().fg(Blue),
            feed: Style::default().fg(White),
            header: Style::default().fg(Magenta),
            tag: Style::default().fg(Blue),
            query: Style::default().fg(Yellow),

            unread_modifier: Modifier::BOLD,
        }
    }
}
