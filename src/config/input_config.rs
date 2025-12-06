use ratatui::crossterm::event::KeyCode;

use crate::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct InputConfig {
    pub scroll_amount: usize,
    pub timeout_millis: u64,
    pub mappings: HashMap<KeySequence, CommandSequence>,
    pub command_line: CommandLineInputConfig,
}

// a macro for pleasure
macro_rules! cmd_mappings {
    [$($key_seq:literal => $($command_seq:literal)*),*,] => {
        vec![$(($key_seq.into(), [$(Command::parse($command_seq, false).unwrap()),*].into()),)*].into_iter().collect()
    };
}

fn generate_default_input_commands() -> HashMap<KeySequence, CommandSequence> {
    cmd_mappings! [
        "up"        => "up",
        "down"      => "down",
        "C-h"       => "left",
        "C-l"       => "right",
        "left"      => "left",
        "right"     => "right",
        "j"         => "down",
        "k"         => "up",
        "space"     => "toggle",
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
        ":"         => "cmd",
        "l"         => "next",
        "h"         => "prev",
        "tab"       => "nextc",
        "backtab"   => "prevc",
        "o"         => "open" "read" "nextunread",
        "O"         => "open unread" "confirm read articles %",
        "J"         => "read" "nextunread",
        "s"         => "sync",
        "r"         => "read",
        "t"         => "cmd tag",
        "R"         => "confirm read %",
        "C-r"       => "cmd read",
        "u"         => "unread",
        "U"         => "confirm unread %",
        "C-u"       => "cmd unread",
        "m"         => "mark",
        "M"         => "confirm mark %",
        "n"         => "unmark",
        "N"         => "confirm unmark %",
        "C-n"       => "cmd unmark",
        "1"         => "show all",
        "2"         => "show unread",
        "3"         => "show marked",
        "z"         => "zen",
        "/"         => "cmd search ",
        "n"         => "searchnext",
        "N"         => "searchprev",
        "="         => "cmd filter ",
        "+ r"       => "filterclear",
        "+ +"       => "filterapply",
        "c w"       => "cmd rename",
        "c d"       => "confirm remove",
        "c x"       => "confirm removeall",
        "c f"       => "cmd feedadd",
        "c c"       => "cmd categoryadd",
        "c u"       => "cmd feedchangeurl",
        "c y"       => "yank",
        "c p"       => "paste after",
        "c P"       => "paste before",
        "c c"       => "cmd tagchangecolor",
    ]
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_amount: 10,
            timeout_millis: 5000,
            mappings: generate_default_input_commands(),
            command_line: CommandLineInputConfig::default(),
        }
    }
}

impl InputConfig {
    pub fn validate(&mut self) -> color_eyre::Result<()> {
        Self::default()
            .mappings
            .into_iter()
            .for_each(|(key_seq, cmd_seq)| {
                self.mappings.entry(key_seq).or_insert(cmd_seq);
            });

        self.mappings
            .iter()
            .filter_map(|(key_seq, command_seq)| command_seq.commands.is_empty().then_some(key_seq))
            .cloned()
            .collect::<Vec<KeySequence>>()
            .into_iter()
            .for_each(|key| {
                self.mappings.remove(&key);
            });

        Ok(())
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
