use ratatui::{
    layout::{Offset, Rect, Size},
    symbols::{
        merge::MergeStrategy,
        scrollbar::{self, Set},
    },
    widgets::{BorderType, Borders},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct BorderTheme {
    pub framing: Framing,
    #[serde(with = "deserialize_border_type_snake_case")]
    pub focused: BorderType,
    #[serde(with = "deserialize_border_type_snake_case")]
    pub unfocused: BorderType,
}

mod deserialize_border_type_snake_case {
    use ratatui::widgets::BorderType;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BorderType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        use BorderType as T;
        Ok(match value.as_str() {
            "plain" => T::Plain,
            "rounded" => T::Rounded,
            "thick" => T::Thick,
            "double" => T::Double,
            "quadrant_inside" => T::QuadrantInside,
            "quadrant_outside" => T::QuadrantOutside,
            _ => {
                return Err(serde::de::Error::unknown_variant(
                    &value,
                    &[
                        "plain",
                        "rounded",
                        "thick",
                        "double",
                        "quadrant_inside",
                        "quadrant_outside",
                    ],
                ));
            }
        })
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Framing {
    #[default]
    Open,
    Closed,
    Connected,
}

impl Default for BorderTheme {
    fn default() -> Self {
        Self {
            framing: Framing::Closed,
            focused: BorderType::Thick,
            unfocused: BorderType::Plain,
        }
    }
}

impl BorderTheme {
    pub fn eff_type(&self, is_focused: bool) -> BorderType {
        if is_focused {
            self.focused
        } else {
            self.unfocused
        }
    }

    pub fn scrollbar_set(&self, is_focused: bool) -> scrollbar::Set<'_> {
        use BorderType as T;
        match self.eff_type(is_focused) {
            T::Plain
            | T::Rounded
            | T::LightDoubleDashed
            | T::LightTripleDashed
            | T::LightQuadrupleDashed => Set {
                begin: "│",
                end: "│",
                track: " ",
                thumb: "│",
            },
            T::Double => Set {
                begin: "║",
                end: "║",
                track: " ",
                thumb: "║",
            },
            T::Thick => Set {
                begin: "┃",
                end: "┃",
                track: " ",
                thumb: "┃",
            },
            T::HeavyDoubleDashed | T::HeavyTripleDashed | T::HeavyQuadrupleDashed => Set {
                begin: "┃",
                end: "┃",
                track: " ",
                thumb: "┃",
            },
            T::QuadrantInside => Set {
                begin: "▌",
                end: "▌",
                track: " ",
                thumb: "▌",
            },
            T::QuadrantOutside => Set {
                begin: "▐",
                end: "▐",
                track: " ",
                thumb: "▐",
            },
        }
    }
}

impl Framing {
    pub fn eff_borders_open(&self, open_border: Borders) -> Borders {
        match self {
            Framing::Open => Borders::ALL.difference(open_border),
            Framing::Closed | Framing::Connected => Borders::ALL,
        }
    }

    pub fn eff_area(&self, open_border: Borders, area: Rect) -> Rect {
        if matches!(self, Framing::Connected) {
            match open_border {
                Borders::RIGHT => area.resize(Size::new(area.width.saturating_add(1), area.height)),
                Borders::BOTTOM => {
                    area.resize(Size::new(area.width, area.height.saturating_add(1)))
                }
                Borders::TOP => area
                    .resize(Size::new(area.width, area.height.saturating_add(1)))
                    .offset(Offset::new(0, -1)),
                Borders::LEFT => area
                    .resize(Size::new(area.width.saturating_add(1), area.height))
                    .offset(Offset::new(-1, 0)),
                _ => unimplemented!("this case should not happen"),
            }
        } else {
            area
        }
    }

    pub fn eff_merge_strategy(&self) -> MergeStrategy {
        match self {
            Framing::Open | Framing::Closed => MergeStrategy::Replace,
            Framing::Connected => MergeStrategy::Fuzzy,
        }
    }
}
