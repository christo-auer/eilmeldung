use std::collections::HashMap;

use include_dir::{Dir, include_dir};
use ratatui::style::Color;

use crate::config::theme::ColorPalette;

const BASE16_THEMES_DIR: Dir = include_dir!("assets/base-16-themes/base16");

#[derive(getset::Getters)]
#[getset(get = "pub")]
pub struct Base16ThemeLibrary {
    theme_for_name: HashMap<String, Base16Theme>,
}

impl Base16ThemeLibrary {
    pub fn load() -> color_eyre::Result<Base16ThemeLibrary> {
        let mut theme_for_name = HashMap::<String, Base16Theme>::new();

        for theme_file in BASE16_THEMES_DIR.files() {
            log::debug!("loading base 16 theme {theme_file:?}");
            let theme: Base16Theme = serde_yaml::from_str(theme_file.contents_utf8().ok_or(
                color_eyre::eyre::eyre!("unable to read contents of {theme_file:?}"),
            )?)?;

            if let Some(file_name) = theme_file
                .path()
                .file_stem()
                .and_then(|os_str| os_str.to_str())
            {
                theme_for_name.entry(file_name.to_owned()).or_insert(theme);
            } else {
                log::warn!("could not load base 16 theme file {theme_file:?}");
            }
        }

        Ok(Self { theme_for_name })
    }
}

#[derive(Debug, serde::Deserialize, getset::CopyGetters, getset::Getters)]
#[allow(non_snake_case, dead_code)]
pub struct Base16Theme {
    #[getset(get = "pub")]
    system: String,
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    author: String,
    #[getset(get = "pub")]
    variant: Base16ThemePolarity,

    #[getset(get = "pub")]
    palette: Base16Palette,
}

#[derive(Debug, Default, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Base16ThemePolarity {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, serde::Deserialize, getset::CopyGetters, getset::Getters)]
#[allow(non_snake_case, dead_code)]
pub struct Base16Palette {
    #[getset(get_copy = "pub")]
    base00: Color,
    #[getset(get_copy = "pub")]
    base01: Color,
    #[getset(get_copy = "pub")]
    base02: Color,
    #[getset(get_copy = "pub")]
    base03: Color,
    #[getset(get_copy = "pub")]
    base04: Color,
    #[getset(get_copy = "pub")]
    base05: Color,
    #[getset(get_copy = "pub")]
    base06: Color,
    #[getset(get_copy = "pub")]
    base07: Color,
    #[getset(get_copy = "pub")]
    base08: Color,
    #[getset(get_copy = "pub")]
    base09: Color,
    #[getset(get_copy = "pub")]
    base0A: Color,
    #[getset(get_copy = "pub")]
    base0B: Color,
    #[getset(get_copy = "pub")]
    base0C: Color,
    #[getset(get_copy = "pub")]
    base0D: Color,
    #[getset(get_copy = "pub")]
    base0E: Color,
    #[getset(get_copy = "pub")]
    base0F: Color,
    #[getset(get_copy = "pub")]
    base10: Option<Color>,
    #[getset(get_copy = "pub")]
    base11: Option<Color>,
    #[getset(get_copy = "pub")]
    base12: Option<Color>,
    #[getset(get_copy = "pub")]
    base13: Option<Color>,
    #[getset(get_copy = "pub")]
    base14: Option<Color>,
    #[getset(get_copy = "pub")]
    base15: Option<Color>,
    #[getset(get_copy = "pub")]
    base16: Option<Color>,
    #[getset(get_copy = "pub")]
    base17: Option<Color>,
}

#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Base16Index {
    Base00,
    Base01,
    Base02,
    Base03,
    Base04,
    Base05,
    Base06,
    Base07,
    Base08,
    Base09,
    Base0A,
    Base0B,
    Base0C,
    Base0D,
    Base0E,
    Base0F,
}

impl std::ops::Index<Base16Index> for Base16Palette {
    type Output = Color;
    fn index(&self, index: Base16Index) -> &Self::Output {
        use Base16Index as I;
        match index {
            I::Base00 => &self.base00,
            I::Base01 => &self.base01,
            I::Base02 => &self.base02,
            I::Base03 => &self.base03,
            I::Base04 => &self.base04,
            I::Base05 => &self.base05,
            I::Base06 => &self.base06,
            I::Base07 => &self.base07,
            I::Base08 => &self.base08,
            I::Base09 => &self.base09,
            I::Base0A => &self.base0A,
            I::Base0B => &self.base0B,
            I::Base0C => &self.base0C,
            I::Base0D => &self.base0D,
            I::Base0E => &self.base0E,
            I::Base0F => &self.base0F,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, getset::CopyGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_copy = "pub")]
pub struct Base16ToColorPaletteMapping {
    pub(crate) background: Base16Index,
    pub(crate) foreground: Base16Index,
    pub(crate) muted: Base16Index,
    pub(crate) highlight: Base16Index,
    pub(crate) flagged: Base16Index,
    pub(crate) accent_primary: Base16Index,
    pub(crate) accent_secondary: Base16Index,
    pub(crate) accent_tertiary: Base16Index,
    pub(crate) accent_quaternary: Base16Index,

    pub(crate) info: Base16Index,
    pub(crate) warning: Base16Index,
    pub(crate) error: Base16Index,
}

impl Base16Palette {
    pub fn as_color_palette(&self, mapping: &Base16ToColorPaletteMapping) -> ColorPalette {
        ColorPalette {
            background: self[mapping.background()],
            foreground: self[mapping.foreground()],
            muted: self[mapping.muted()],
            highlight: self[mapping.highlight()],
            flagged: self[mapping.flagged()],
            accent_primary: self[mapping.accent_primary()],
            accent_secondary: self[mapping.accent_secondary()],
            accent_tertiary: self[mapping.accent_tertiary()],
            accent_quaternary: self[mapping.accent_quaternary()],

            info: self[mapping.info()],
            warning: self[mapping.warning()],
            error: self[mapping.error()],
        }
    }
}

impl Default for Base16ToColorPaletteMapping {
    fn default() -> Self {
        Self::default_for_polarity(Base16ThemePolarity::Dark)
    }
}

impl Base16ToColorPaletteMapping {
    pub(crate) fn default_for_polarity(
        polarity: Base16ThemePolarity,
    ) -> Base16ToColorPaletteMapping {
        use Base16Index as I;
        match polarity {
            Base16ThemePolarity::Dark => Self {
                background: I::Base00,
                foreground: I::Base05,
                muted: I::Base02,
                highlight: I::Base0A,
                flagged: I::Base08,
                accent_primary: I::Base0B,
                accent_secondary: I::Base0C,
                accent_tertiary: I::Base0D,
                accent_quaternary: I::Base0E,

                info: I::Base0B,
                warning: I::Base0A,
                error: I::Base08,
            },
            Base16ThemePolarity::Light => Self {
                background: I::Base00,
                foreground: I::Base07,
                muted: I::Base03,
                highlight: I::Base02,
                flagged: I::Base08,
                accent_primary: I::Base0B,
                accent_secondary: I::Base0C,
                accent_tertiary: I::Base0D,
                accent_quaternary: I::Base0E,

                info: I::Base0B,
                warning: I::Base0A,
                error: I::Base08,
            },
        }
    }
}
