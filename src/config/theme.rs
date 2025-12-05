use ratatui::style::{Color, Modifier, Style, Stylize};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
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
    pub info_color: Color,
    pub warning_color: Color,
    pub error_color: Color,

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
        use Color as C;
        let background_color = C::Black;
        let muted_color = C::DarkGray;
        let normal_color = C::White;
        let normal_color_2 = C::Magenta;
        let highlight_color = C::Yellow;
        let accent_color = C::Magenta;
        let accent_color_2 = C::Blue;
        let accent_color_3 = C::Cyan;
        let accent_color_4 = C::Yellow;

        let info_color = C::Magenta;
        let warning_color = C::Yellow;
        let error_color = C::Red;

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

            warning_color,
            info_color,
            error_color,

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
