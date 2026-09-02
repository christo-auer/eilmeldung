use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use include_dir::{Dir, include_dir};
use ratatui::style::Color;

use crate::config::theme::ColorPalette;

const BASE16_THEMES_DIR: Dir = include_dir!("assets/base16-schemes");

#[derive(Clone, Debug, serde::Deserialize, getset::CopyGetters, getset::Getters)]
#[allow(non_snake_case, dead_code)]
pub struct Base16Theme {
    #[getset(get = "pub")]
    system: Option<String>,
    #[getset(get = "pub")]
    name: Option<String>,
    #[getset(get = "pub")]
    author: Option<String>,
    #[getset(get = "pub")]
    variant: Base16ThemePolarity,

    #[getset(get = "pub")]
    palette: Base16Palette,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Base16ThemeEntry {
    Custom {
        name: String,
        path: PathBuf,
    },
    Library {
        name: String,
        file: include_dir::File<'static>,
    },
}

impl PartialOrd for Base16ThemeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Base16ThemeEntry as E;
        Some(match (self, other) {
            (E::Custom { .. }, E::Library { .. }) => Ordering::Less,
            (E::Library { .. }, E::Custom { .. }) => Ordering::Greater,
            (e1, e2) => e1.name().cmp(&e2.name()),
        })
    }
}

impl Base16ThemeEntry {
    pub fn custom(name: &str, path: PathBuf) -> Self {
        Base16ThemeEntry::Custom {
            name: name.to_owned(),
            path,
        }
    }
    pub fn library(name: &str, file: include_dir::File<'static>) -> Self {
        Base16ThemeEntry::Library {
            name: name.to_owned(),
            file,
        }
    }

    pub fn load_theme(&self) -> color_eyre::eyre::Result<Base16Theme> {
        let contents = match self {
            Base16ThemeEntry::Custom { name: _, path } => std::fs::read_to_string(path)?,
            Base16ThemeEntry::Library { name: _, file } => file.contents_utf8().unwrap().to_owned(),
        };
        serde_yaml::from_str(&contents).map_err(|serde_error| {
            color_eyre::eyre::eyre!("could not parse theme {}: {serde_error}", self.name())
        })
    }

    pub fn name(&self) -> String {
        match self {
            Base16ThemeEntry::Custom { name, .. } => name.to_owned(),
            Base16ThemeEntry::Library { name, .. } => name.to_owned(),
        }
    }
}

impl Base16Theme {
    pub fn available_themes(
        custom_themes_path: &Path,
    ) -> color_eyre::Result<Vec<Base16ThemeEntry>> {
        let mut themes: Vec<Base16ThemeEntry> = if custom_themes_path.exists() {
            std::fs::read_dir(custom_themes_path)?
                .filter_map(|dir_entry| {
                    dir_entry
                        .ok()
                        .and_then(|entry| {
                            entry.path().to_string_lossy().ends_with(".yaml").then(|| {
                                entry.path().file_stem().map(|stem| {
                                    Base16ThemeEntry::custom(
                                        &stem.to_string_lossy(),
                                        custom_themes_path.join(entry.path()),
                                    )
                                })
                            })
                        })
                        .flatten()
                })
                .collect()
        } else {
            Default::default()
        };

        themes.extend(BASE16_THEMES_DIR.files().filter_map(|entry| {
            entry
                .path()
                .to_string_lossy()
                .ends_with(".yaml")
                .then(|| {
                    entry.path().file_stem().map(|stem| {
                        Base16ThemeEntry::library(&stem.to_string_lossy(), entry.to_owned())
                    })
                })
                .flatten()
        }));

        Ok(themes)
    }
}

#[derive(Debug, Default, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Base16ThemePolarity {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug, serde::Deserialize, getset::CopyGetters, getset::Getters)]
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
#[serde(rename_all = "lowercase")]
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
