use std::fmt::Display;
use std::str::FromStr;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Key {
    Ctrl(KeyCode),
    Alt(KeyCode),
    Shift(KeyCode),
    Just(KeyCode),
    Unknown,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default)]
pub struct KeySequence {
    pub keys: Vec<Key>,
}

impl Key {
    fn parse_key_code(s: &str) -> color_eyre::Result<KeyCode> {
        Ok(match s {
            "enter" => KeyCode::Enter,
            "space" => KeyCode::Char(' '),
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "backspace" => KeyCode::Backspace,
            "esc" => KeyCode::Esc,

            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,

            "insert" => KeyCode::Insert,
            "delete" => KeyCode::Delete,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "page_up" => KeyCode::PageUp,
            "page_down" => KeyCode::PageDown,

            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),

            _ => {
                if let Some(first_char) = s.chars().next()
                    && first_char != ' '
                {
                    KeyCode::Char(first_char)
                } else {
                    return Err(color_eyre::Report::msg(format!("unable to parse key {s}")));
                }
            }
        })
    }
}

impl FromStr for Key {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars().collect::<Vec<_>>();
        if s.len() > 2 && chars[1] == '-' && chars[2] != ' ' {
            let mut chars = chars.into_iter();
            let prefix = chars.next().unwrap(); // <- safe as s.len()>2
            chars.next(); // skip -

            let key = Key::parse_key_code(&chars.collect::<String>());

            match prefix {
                'C' => Ok(key.map(Key::Ctrl)?),
                'M' => Ok(key.map(Key::Alt)?),
                'S' => Ok(key.map(Key::Shift)?),
                _ => Err(color_eyre::Report::msg(format!(
                    "unable to parse key from `{s}``"
                ))),
            }
        } else {
            Ok(Self::parse_key_code(s).map(Key::Just)?)
        }
    }
}

fn key_code_to_string(key_code: KeyCode) -> Option<String> {
    Some(match key_code {
        KeyCode::Char(char) => {
            if char == ' ' {
                "space".to_string()
            } else {
                char.to_string()
            }
        }
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::BackTab => "backtab".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Esc => "esc".to_string(),

        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),

        KeyCode::Insert => "insert".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "page_up".to_string(),
        KeyCode::PageDown => "page_down".to_string(),

        KeyCode::F(1) => "f1".to_string(),
        KeyCode::F(2) => "f2".to_string(),
        KeyCode::F(3) => "f3".to_string(),
        KeyCode::F(4) => "f4".to_string(),
        KeyCode::F(5) => "f5".to_string(),
        KeyCode::F(6) => "f6".to_string(),
        KeyCode::F(7) => "f7".to_string(),
        KeyCode::F(8) => "f8".to_string(),
        KeyCode::F(9) => "f9".to_string(),
        KeyCode::F(10) => "f10".to_string(),
        KeyCode::F(11) => "f11".to_string(),
        KeyCode::F(12) => "f12".to_string(),

        _ => {
            return None;
        }
    })
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Key::*;
        match *self {
            Ctrl(key_code) => write!(
                f,
                "C-{}",
                key_code_to_string(key_code).unwrap_or("unknown key".into())
            ),
            Alt(key_code) => write!(
                f,
                "M-{}",
                key_code_to_string(key_code).unwrap_or("unknown key".into())
            ),
            Shift(key_code) => write!(
                f,
                "S-{}",
                key_code_to_string(key_code).unwrap_or("unknown key".into())
            ),
            Just(key_code) => write!(
                f,
                "{}",
                key_code_to_string(key_code).unwrap_or("unknown key".into())
            ),
            Unknown => write!(f, "unknown key"),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Self::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl FromStr for KeySequence {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = s.split(' ').collect::<Vec<_>>();

        keys.into_iter()
            .map(Key::from_str)
            .map(|key| key.ok())
            .collect::<Option<Vec<Key>>>()
            .map(|keys| Self { keys })
            .ok_or(color_eyre::Report::msg(format!(
                "unable to parse sequence {s}"
            )))
    }
}

// silently fails! use only if completely sure this does not fail!
impl From<&str> for KeySequence {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap_or_default()
    }
}

impl From<KeyEvent> for Key {
    fn from(key_event: KeyEvent) -> Self {
        let modifiers = key_event.modifiers;
        // modifiers &= !KeyModifiers::SHIFT; // remove SHIFT bit

        match modifiers {
            KeyModifiers::NONE => Key::Just(key_event.code),
            KeyModifiers::ALT => Key::Alt(key_event.code),
            KeyModifiers::CONTROL => Key::Ctrl(key_event.code),
            KeyModifiers::SHIFT => match key_event.code {
                code @ KeyCode::Char(_) => Key::Just(code),
                code => Key::Shift(code),
            },
            _ => Key::Unknown,
        }
    }
}

impl std::fmt::Display for KeySequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.keys
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl KeySequence {
    pub fn is_prefix_of(&self, other: &KeySequence) -> bool {
        if self.keys.len() > other.keys.len() {
            return false;
        }

        other
            .keys
            .iter()
            .zip(self.keys.iter())
            .all(|(key_a, key_b)| *key_a == *key_b)
    }
}

impl<'de> serde::de::Deserialize<'de> for KeySequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        KeySequence::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
