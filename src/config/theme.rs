use ratatui::style::{Color, Modifier, Style, Stylize};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Theme {
    pub feeds_list_focus_width_percent: u16,
    pub articles_list_focused_width_percent: u16,
    pub articles_list_height_lines: u16,

    pub background_color: Color,
    pub muted_color: Color,
    pub normal_color: Color,
    pub highlight_color: Color,
    pub accent_color: Color,
    pub accent_color_2: Color,
    pub accent_color_3: Color,
    pub accent_color_4: Color,

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
    pub command_input: Style,
    pub inactive: Style,

    pub border_style: Style,
    pub focused_border_style: Style,

    pub unread_modifier: Modifier,
}

impl Default for Theme {
    fn default() -> Self {
        use Color::*;
        let background_color = Black;
        let muted_color = DarkGray;
        let normal_color = White;
        let normal_color_2 = Magenta;
        let highlight_color = Yellow;
        let accent_color = Magenta;
        let accent_color_2 = Blue;
        let accent_color_3 = Cyan;
        let accent_color_4 = Yellow;

        let info_color = Magenta;
        let warning_color = Yellow;
        let error_color = Red;

        Self {
            feeds_list_focus_width_percent: 33,
            articles_list_focused_width_percent: 67,
            articles_list_height_lines: 6,

            background_color,
            muted_color,
            normal_color,
            highlight_color,
            accent_color,
            accent_color_2,
            accent_color_3,
            accent_color_4,

            paragraph: Style::default().fg(normal_color),
            article: Style::default().fg(normal_color),
            header: Style::default().fg(normal_color_2),
            feed: Style::default().fg(accent_color),
            category: Style::default().fg(accent_color_2),
            article_highlighted: Style::default().fg(highlight_color).bold(),
            tag: Style::default().fg(accent_color_3),
            query: Style::default().fg(accent_color_4),
            statusbar: Style::default().fg(normal_color_2).bg(muted_color).bold(),
            tooltip_info: Style::default().fg(background_color).bg(info_color),
            tooltip_warning: Style::default()
                .fg(background_color)
                .bg(warning_color)
                .bold(),

            tooltip_error: Style::default().fg(background_color).bg(error_color).bold(),
            command_input: Style::default().bg(muted_color).fg(normal_color),
            inactive: Style::default().fg(muted_color),

            border_style: Style::default().fg(muted_color),
            focused_border_style: Style::default().fg(accent_color),

            unread_modifier: Modifier::BOLD,
        }
    }
}
