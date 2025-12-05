use std::str::FromStr;

use ratatui::{
    style::{Color, Modifier, ParseColorError, Style, Stylize},
    widgets::BorderType,
};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ColorPalette {
    background: Color,
    foreground: Color,
    muted: Color,
    highlight: Color,
    accent_primary: Color,
    accent_secondary: Color,
    accent_tertiary: Color,
    accent_quaternary: Color,

    info: Color,
    warning: Color,
    error: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        use Color as C;
        Self {
            background: C::Black,
            foreground: C::White,
            muted: C::DarkGray,
            highlight: C::Yellow,
            accent_primary: C::Magenta,
            accent_secondary: C::Blue,
            accent_tertiary: C::Cyan,
            accent_quaternary: C::Yellow,

            info: C::Magenta,
            warning: C::Yellow,
            error: C::Red,
        }
    }
}

#[derive(Debug, Copy, Default, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleColor {
    #[default]
    None,

    Background,
    Foreground,
    Muted,
    Highlight,
    AccentPrimary,
    AccentSecondary,
    AccentTertiary,
    AccentQuaternary,
    Info,
    Warning,
    Error,

    Custom(Color),
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct ComponentStyle {
    fg: StyleColor,
    bg: StyleColor,

    #[serde(default)]
    mods: Vec<Modifier>,
}

impl ComponentStyle {
    fn fg(self, fg: StyleColor) -> Self {
        Self { fg, ..self }
    }

    fn bg(self, bg: StyleColor) -> Self {
        Self { bg, ..self }
    }

    fn mods(self, mods: &[Modifier]) -> Self {
        Self {
            mods: mods.to_owned(),
            ..self
        }
    }

    fn modifiers(&self) -> Modifier {
        self.mods
            .iter()
            .fold(Modifier::default(), |mut modifiers, modifier| {
                modifiers |= *modifier;
                modifiers
            })
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct StyleSet {
    header: ComponentStyle,
    paragraph: ComponentStyle,
    article: ComponentStyle,
    article_highlighted: ComponentStyle,
    feed: ComponentStyle,
    category: ComponentStyle,
    tag: ComponentStyle,
    query: ComponentStyle,
    yanked: ComponentStyle,

    border: ComponentStyle,
    border_focused: ComponentStyle,
    statusbar: ComponentStyle,
    command_input: ComponentStyle,
    inactive: ComponentStyle,

    tooltip_info: ComponentStyle,
    tooltip_warning: ComponentStyle,
    tooltip_error: ComponentStyle,

    unread_modifier: Modifier,
}

impl Default for StyleSet {
    fn default() -> Self {
        use Modifier as M;
        use StyleColor as C;
        Self {
            header: ComponentStyle::default().fg(C::AccentPrimary),
            paragraph: ComponentStyle::default().fg(C::Foreground),
            article: ComponentStyle::default().fg(C::Foreground),
            article_highlighted: ComponentStyle::default().fg(C::Highlight).mods(&[M::BOLD]),
            feed: ComponentStyle::default().fg(C::AccentPrimary),
            category: ComponentStyle::default().fg(C::AccentSecondary),
            tag: ComponentStyle::default().fg(C::AccentTertiary),
            query: ComponentStyle::default().fg(C::AccentQuaternary),
            yanked: ComponentStyle::default()
                .fg(C::Highlight)
                .mods(&[M::REVERSED]),

            border: ComponentStyle::default().fg(C::Muted),
            border_focused: ComponentStyle::default().fg(C::AccentPrimary),
            statusbar: ComponentStyle::default()
                .fg(C::Background)
                .bg(C::AccentPrimary),
            command_input: ComponentStyle::default().fg(C::Foreground).bg(C::Muted),
            inactive: ComponentStyle::default().fg(C::Muted),

            tooltip_info: ComponentStyle::default().fg(C::Background).bg(C::Info),
            tooltip_warning: ComponentStyle::default().fg(C::Background).bg(C::Warning),
            tooltip_error: ComponentStyle::default().fg(C::Background).bg(C::Error),

            unread_modifier: Modifier::BOLD,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct Theme {
    color_palette: ColorPalette,
    style_set: StyleSet,
}

macro_rules! component_funs {
    {$($prop:ident),*} => {
        $(pub fn $prop(&self) -> Style {
            self.to_style(&self.style_set.$prop)
        })*
    };
}

impl Theme {
    pub fn color(&self, style_color: StyleColor) -> Color {
        use Color as C;
        use StyleColor as SC;
        match style_color {
            SC::None => C::default(),
            SC::Background => self.color_palette.background,
            SC::Foreground => self.color_palette.foreground,
            SC::Muted => self.color_palette.muted,
            SC::Highlight => self.color_palette.highlight,
            SC::AccentPrimary => self.color_palette.accent_primary,
            SC::AccentSecondary => self.color_palette.accent_secondary,
            SC::AccentTertiary => self.color_palette.accent_tertiary,
            SC::AccentQuaternary => self.color_palette.accent_quaternary,
            SC::Info => self.color_palette.info,
            SC::Warning => self.color_palette.warning,
            SC::Error => self.color_palette.error,
            SC::Custom(color) => color,
        }
    }

    pub fn to_style(&self, component_style: &ComponentStyle) -> Style {
        Style::new()
            .fg(self.color(component_style.fg))
            .bg(self.color(component_style.bg))
            .add_modifier(component_style.modifiers())
    }

    pub fn unread_modifier(&self) -> Modifier {
        self.style_set.unread_modifier
    }

    component_funs! {
      header,
      paragraph,
      article,
      article_highlighted,
      feed,
      category,
      tag,
      query,
      yanked,
      border,
      border_focused,
      statusbar,
      command_input,
      inactive,
      tooltip_info,
      tooltip_warning,
      tooltip_error
    }
}

// impl Default for Theme {
//     fn default() -> Self {
//         use Color as C;
//         let background_color = C::Black;
//         let muted_color = C::DarkGray;
//         let normal_color = C::White;
//         let normal_color_2 = C::Magenta;
//         let highlight_color = C::Yellow;
//         let accent_color = C::Magenta;
//         let accent_color_2 = C::Blue;
//         let accent_color_3 = C::Cyan;
//         let accent_color_4 = C::Yellow;
//
//         let info_color = C::Magenta;
//         let warning_color = C::Yellow;
//         let error_color = C::Red;
//
//         Self {
//             background_color,
//             muted_color,
//             normal_color,
//             highlight_color,
//             accent_color,
//             accent_color_2,
//             accent_color_3,
//             accent_color_4,
//
//             warning_color,
//             info_color,
//             error_color,
//
//             paragraph: Style::default().fg(normal_color),
//             article: Style::default().fg(normal_color),
//             header: Style::default().fg(normal_color_2),
//             feed: Style::default().fg(accent_color),
//             category: Style::default().fg(accent_color_2),
//             article_highlighted: Style::default().fg(highlight_color).bold(),
//             tag: Style::default().fg(accent_color_3),
//             query: Style::default().fg(accent_color_4),
//             statusbar: Style::default().fg(normal_color_2).bg(muted_color).bold(),
//             tooltip_info: Style::default().fg(background_color).bg(info_color),
//             tooltip_warning: Style::default()
//                 .fg(background_color)
//                 .bg(warning_color)
//                 .bold(),
//
//             tooltip_error: Style::default().fg(background_color).bg(error_color).bold(),
//             command_input: Style::default().bg(muted_color).fg(normal_color),
//             inactive: Style::default().fg(muted_color),
//
//             border_style: Style::default().fg(muted_color),
//             focused_border_style: Style::default().fg(accent_color),
//
//             unread_modifier: Modifier::BOLD,
//         }
//     }
// }
