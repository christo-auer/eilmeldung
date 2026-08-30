use crate::{config::base16_palette::Base16ThemePolarity, prelude::*};
use std::str::FromStr;

use getset::Getters;
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, serde::Deserialize, Getters)]
#[serde(default)]
#[getset(get = "pub")]
pub struct ColorPalette {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) muted: Color,
    pub(crate) highlight: Color,
    pub(crate) flagged: Color,
    pub(crate) accent_primary: Color,
    pub(crate) accent_secondary: Color,
    pub(crate) accent_tertiary: Color,
    pub(crate) accent_quaternary: Color,

    pub(crate) info: Color,
    pub(crate) warning: Color,
    pub(crate) error: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        use Color as C;
        Self {
            background: C::Reset,
            foreground: C::White,
            muted: C::DarkGray,
            highlight: C::Yellow,
            flagged: C::Red,
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

#[derive(Debug, Copy, Default, Clone)]
pub enum StyleColor {
    #[default]
    None,

    Background,
    Foreground,
    Muted,
    Highlight,
    Flagged,
    AccentPrimary,
    AccentSecondary,
    AccentTertiary,
    AccentQuaternary,
    Info,
    Warning,
    Error,

    Custom(Color),
}

impl<'de> serde::de::Deserialize<'de> for StyleColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        use StyleColor as C;

        Ok(match s.trim() {
            "none" => C::None,
            "background" => C::Background,
            "foreground" => C::Foreground,
            "muted" => C::Muted,
            "highlight" => C::Highlight,
            "flagged" => C::Flagged,
            "accent_primary" => C::AccentPrimary,
            "accent_secondary" => C::AccentSecondary,
            "accent_tertiary" => C::AccentTertiary,
            "accent_quaternary" => C::AccentQuaternary,
            "info" => C::Info,
            "warning" => C::Warning,
            "error" => C::Error,
            s => C::Custom(
                Color::from_str(s)
                    .map_err(|_| serde::de::Error::custom(format!("unable to parse color: {s}")))?,
            ),
        })
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct ComponentStyle {
    #[serde(default)]
    fg: StyleColor,

    #[serde(default)]
    bg: StyleColor,

    #[serde(default)]
    mods: Vec<StyleModifier>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleModifier {
    Bold,
    Dim,
    Italic,
    Underlined,
    SlowBlink,
    RapidBlink,
    Reversed,
    Hidden,
    CrossedOut,
}

impl StyleModifier {
    fn to_modifier(self) -> Modifier {
        use StyleModifier as S;
        match self {
            S::Bold => Modifier::BOLD,
            S::Dim => Modifier::DIM,
            S::Italic => Modifier::ITALIC,
            S::Underlined => Modifier::UNDERLINED,
            S::SlowBlink => Modifier::SLOW_BLINK,
            S::RapidBlink => Modifier::RAPID_BLINK,
            S::Reversed => Modifier::REVERSED,
            S::Hidden => Modifier::HIDDEN,
            S::CrossedOut => Modifier::CROSSED_OUT,
        }
    }
}

impl ComponentStyle {
    fn fg(self, fg: StyleColor) -> Self {
        Self { fg, ..self }
    }

    fn bg(self, bg: StyleColor) -> Self {
        Self { bg, ..self }
    }

    fn mods(self, mods: &[StyleModifier]) -> Self {
        Self {
            mods: mods.to_owned(),
            ..self
        }
    }

    fn modifiers(&self) -> Modifier {
        self.mods
            .iter()
            .fold(Modifier::default(), |mut modifiers, modifier| {
                modifiers |= modifier.to_modifier();
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

    unread: ComponentStyle,
    unread_count: ComponentStyle,
    marked_count: ComponentStyle,
    read: ComponentStyle,
    selected: ComponentStyle,
    highlighted: ComponentStyle,
    flagged: ComponentStyle,
}

impl Default for StyleSet {
    fn default() -> Self {
        use StyleColor as C;
        use StyleModifier as M;
        Self {
            header: ComponentStyle::default().fg(C::AccentPrimary),
            paragraph: ComponentStyle::default().fg(C::Foreground),
            article: ComponentStyle::default().fg(C::Foreground),
            feed: ComponentStyle::default().fg(C::AccentPrimary),
            category: ComponentStyle::default().fg(C::AccentSecondary),
            tag: ComponentStyle::default().fg(C::AccentTertiary),
            query: ComponentStyle::default().fg(C::AccentQuaternary),
            yanked: ComponentStyle::default()
                .fg(C::Highlight)
                .mods(&[M::Reversed]),

            border: ComponentStyle::default().fg(C::Muted),
            border_focused: ComponentStyle::default().fg(C::AccentPrimary),
            statusbar: ComponentStyle::default()
                .fg(C::AccentPrimary)
                .mods(&[M::Reversed]),
            command_input: ComponentStyle::default().fg(C::Foreground).bg(C::Muted),
            inactive: ComponentStyle::default().fg(C::Muted),

            tooltip_info: ComponentStyle::default().fg(C::Info).mods(&[M::Reversed]),
            tooltip_warning: ComponentStyle::default()
                .fg(C::Warning)
                .mods(&[M::Reversed]),
            tooltip_error: ComponentStyle::default().fg(C::Error).mods(&[M::Reversed]),

            unread: ComponentStyle::default().mods(&[M::Bold]),
            read: ComponentStyle::default().mods(&[M::Dim]),
            selected: ComponentStyle::default().mods(&[M::Reversed]),
            highlighted: ComponentStyle::default()
                .fg(C::Highlight)
                .mods(&[M::Italic]),

            flagged: ComponentStyle::default().fg(C::Flagged),

            unread_count: ComponentStyle::default().mods(&[M::Italic]),
            marked_count: ComponentStyle::default().mods(&[M::Italic]),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, Getters)]
#[serde(default)]
#[getset(get = "pub")]
pub struct Theme {
    preset: Option<String>,
    color_palette: ColorPalette,
    style_set: StyleSet,
    base16_dark: Base16ToColorPaletteMapping,
    base16_light: Base16ToColorPaletteMapping,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            preset: Default::default(),
            color_palette: Default::default(),
            style_set: Default::default(),
            base16_dark: Base16ToColorPaletteMapping::default_for_polarity(
                Base16ThemePolarity::Dark,
            ),
            base16_light: Base16ToColorPaletteMapping::default_for_polarity(
                Base16ThemePolarity::Light,
            ),
        }
    }
}

macro_rules! component_funs {
    {$($prop:ident),*} => {
        $(pub fn $prop(&self) -> Style {
            self.to_style(&self.style_set.$prop)
        })*
    };
}

macro_rules! patch_funs {
    {$($prop:ident),*} => {
        $(pub fn $prop(&self, style: &Style) -> Style {
            style.patch(self.to_style(&self.style_set.$prop))
        })*
    };
}

impl Theme {
    pub fn color(&self, style_color: StyleColor) -> Option<Color> {
        use StyleColor as SC;
        Some(match style_color {
            SC::None => return None,
            SC::Background => self.color_palette.background,
            SC::Foreground => self.color_palette.foreground,
            SC::Muted => self.color_palette.muted,
            SC::Highlight => self.color_palette.highlight,
            SC::Flagged => self.color_palette.flagged,
            SC::AccentPrimary => self.color_palette.accent_primary,
            SC::AccentSecondary => self.color_palette.accent_secondary,
            SC::AccentTertiary => self.color_palette.accent_tertiary,
            SC::AccentQuaternary => self.color_palette.accent_quaternary,
            SC::Info => self.color_palette.info,
            SC::Warning => self.color_palette.warning,
            SC::Error => self.color_palette.error,
            SC::Custom(color) => color,
        })
    }

    pub fn eff_border(&self, is_focused: bool) -> Style {
        if is_focused {
            self.border_focused()
        } else {
            self.border()
        }
    }

    pub fn to_style(&self, component_style: &ComponentStyle) -> Style {
        Style {
            fg: self.color(component_style.fg),
            bg: self.color(component_style.bg),
            add_modifier: component_style.modifiers(),
            ..Default::default()
        }
    }

    pub fn validate(&mut self) -> color_eyre::Result<()> {
        let Some(preset) = self.preset.as_ref() else {
            return Ok(());
        };

        let library = Base16ThemeLibrary::load()?;

        let Some(base16_theme) = library.theme_for_name().get(preset) else {
            return Err(color_eyre::eyre::eyre!(
                "base 16 theme preset with name {preset} not found. check value of `preset`"
            ));
        };

        self.color_palette =
            base16_theme
                .palette()
                .as_color_palette(match base16_theme.variant() {
                    Base16ThemePolarity::Dark => &self.base16_dark,
                    Base16ThemePolarity::Light => &self.base16_light,
                });
        Ok(())
    }

    patch_funs! {
        unread,
        read,
        selected,
        highlighted,
        flagged
    }

    component_funs! {
      header,
      paragraph,
      article,
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
      tooltip_error,
      unread_count,
      marked_count
    }
}
