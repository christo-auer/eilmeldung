use std::{fmt::Display, str::FromStr};

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use serde::Deserialize;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum MouseInput {
    Ctrl(MouseEventKind),
    Shift(MouseEventKind),
    Alt(MouseEventKind),
    Just(MouseEventKind),
    #[default]
    Unknown,
}

impl MouseInput {
    pub fn parse_mouse_event_kind(value: &str) -> color_eyre::Result<MouseEventKind> {
        Ok(match value {
            "left" => MouseEventKind::Down(MouseButton::Left),
            "right" => MouseEventKind::Down(MouseButton::Right),
            "middle" => MouseEventKind::Down(MouseButton::Middle),
            "scroll_up" | "up" => MouseEventKind::ScrollUp,
            "scroll_down" | "down" => MouseEventKind::ScrollDown,
            "scroll_left" => MouseEventKind::ScrollLeft,
            "scroll_right" => MouseEventKind::ScrollRight,
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "unable to parse mouse input `{value}`"
                ));
            }
        })
    }

    pub fn kind(self) -> Option<MouseEventKind> {
        use MouseInput as I;
        match self {
            I::Ctrl(kind) | I::Shift(kind) | I::Alt(kind) | I::Just(kind) => Some(kind),
            I::Unknown => None,
        }
    }
}

impl From<&str> for MouseInput {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap_or_default()
    }
}

impl FromStr for MouseInput {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars().collect::<Vec<_>>();
        if s.len() > 2 && chars[1] == '-' {
            let mut chars = chars.into_iter();
            let prefix = chars.next().unwrap(); // <- safe as s.len()>2
            chars.next(); // skip -

            let mouse_input = MouseInput::parse_mouse_event_kind(chars.collect::<String>().trim());

            match prefix {
                'C' => Ok(mouse_input.map(MouseInput::Ctrl)?),
                'M' => Ok(mouse_input.map(MouseInput::Alt)?),
                'S' => Ok(mouse_input.map(MouseInput::Shift)?),
                _ => Err(color_eyre::Report::msg(format!(
                    "unable to parse mouse input from `{s}``"
                ))),
            }
        } else {
            Ok(Self::parse_mouse_event_kind(s).map(MouseInput::Just)?)
        }
    }
}

fn mouse_event_kind_to_string(kind: MouseEventKind) -> Option<String> {
    use MouseEventKind as K;
    Some(
        match kind {
            K::Down(MouseButton::Left) => "left",
            K::Down(MouseButton::Right) => "right",
            K::Down(MouseButton::Middle) => "middle",
            K::ScrollDown => "scroll_down",
            K::ScrollUp => "scroll_up",
            K::ScrollLeft => "scroll_left",
            K::ScrollRight => "scroll_right",
            _ => return None,
        }
        .into(),
    )
}

impl From<MouseEvent> for MouseInput {
    fn from(value: MouseEvent) -> Self {
        match value.modifiers {
            KeyModifiers::CONTROL => MouseInput::Ctrl(value.kind),
            KeyModifiers::SHIFT => MouseInput::Shift(value.kind),
            KeyModifiers::ALT => MouseInput::Alt(value.kind),
            KeyModifiers::NONE => MouseInput::Just(value.kind),
            _ => MouseInput::Unknown,
        }
    }
}

impl Display for MouseInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MouseInput as I;
        match *self {
            I::Ctrl(kind) => write!(
                f,
                "C-{}",
                mouse_event_kind_to_string(kind).unwrap_or("unknown input".into())
            ),
            I::Shift(kind) => write!(
                f,
                "S-{}",
                mouse_event_kind_to_string(kind).unwrap_or("unknown input".into())
            ),
            I::Alt(kind) => write!(
                f,
                "M-{}",
                mouse_event_kind_to_string(kind).unwrap_or("unknown input".into())
            ),
            I::Just(kind) => write!(
                f,
                "{}",
                mouse_event_kind_to_string(kind).unwrap_or("unknown input".into())
            ),
            I::Unknown => write!(f, "unknown mouse input"),
        }
    }
}

impl<'de> Deserialize<'de> for MouseInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
