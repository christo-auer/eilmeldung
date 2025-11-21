use ratatui::crossterm::event::KeyCode;

use crate::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InputConfig {
    pub scroll_amount: usize,
    pub input_timeout_millis: u64,
    pub input_commands: HashMap<KeySequence, CommandSequence>,
    pub command_line: CommandLineInputConfig,
}

// a macro for pleasure
macro_rules! cmd_mappings {
    [$($key_seq:literal => $($command_seq:literal)*),*] => {
        vec![$(($key_seq.into(), [$($command_seq),*].into()),)*].into_iter().collect()
    };
}

fn generate_default_input_commands() -> HashMap<KeySequence, CommandSequence> {
    cmd_mappings! [
        "j"         => "down",
        "k"         => "up",
        "h"         => "left",
        "l"         => "right",
        "C-f"       => "pagedown",
        "C-b"       => "pagedown",
        "g g"       => "gotofirst",
        "G"         => "gotolast",
        "q"         => "confirm quit",
        "C-c"       => "quit",
        "s"         => "scrape",
        "g f"       => "focus feeds",
        "g a"       => "focus articles",
        "g c"       => "focus content",
        ":"         => ":",
        "space"     => "next",
        "backspace" => "prev",
        "tab"       => "nextc",
        "backtab"   => "prevc",
        "o"         => "open" "read" "nextu",
        "O"         => "open unread" "confirm read %",
        "J"         => "read" "nextu",
        "s"         => "sync",
        "r"         => "read",
        "R"         => "confirm read %",
        "C-r"       => ": read",
        "u"         => "unread",
        "U"         => "confirm unread %",
        "C-u"       => ": unread",
        "m"         => "mark",
        "M"         => "confirm mark %",
        "n"         => "unmark",
        "N"         => "confirm unmark %",
        "C-n"       => ": unmark",
        "1"         => "scope all",
        "2"         => "scope unread",
        "3"         => "scope marked",
        "z"         => "zen",
        "/"         => ": / ",
        "n"         => "/next",
        "N"         => "/prev",
        "="         => ": = ",
        "+ r"       => "= clear",
        "+ +"       => "= apply",
        "c w"       => ": rename",
        "d d"       => "confirm remove",
        "D D"       => "confirm removeall",
        "c u"       => ": feedchangeurl"
    ]
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_amount: 10,
            input_timeout_millis: 5000,
            input_commands: generate_default_input_commands(),
            command_line: CommandLineInputConfig::default(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandLineInputConfig {
    pub history_previous: Vec<Key>,
    pub history_next: Vec<Key>,
    pub clear: Vec<Key>,
    pub abort: Vec<Key>,
    pub submit: Vec<Key>,
}

impl Default for CommandLineInputConfig {
    fn default() -> Self {
        Self {
            history_previous: [Key::Ctrl(KeyCode::Char('p')), Key::Just(KeyCode::Up)].into(),
            history_next: [Key::Ctrl(KeyCode::Char('n')), Key::Just(KeyCode::Down)].into(),
            clear: [Key::Ctrl(KeyCode::Char('g'))].into(),
            abort: [Key::Just(KeyCode::Esc)].into(),
            submit: [Key::Just(KeyCode::Enter)].into(),
        }
    }
}
