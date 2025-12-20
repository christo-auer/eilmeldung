use indexmap::IndexMap;

use crate::prelude::*;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct InputConfig {
    pub scroll_amount: usize,
    pub timeout_millis: u64,
    pub mappings: IndexMap<KeySequence, CommandSequence>,
}

// a macro for pleasure
macro_rules! cmd_mappings {
    [$($key_seq:literal => $($command_seq:literal)*),*,] => {
        vec![$(($key_seq.into(), [$(Command::parse($command_seq, false).unwrap()),*].into()),)*].into_iter().collect()
    };
}

fn generate_default_input_commands() -> IndexMap<KeySequence, CommandSequence> {
    cmd_mappings! [
        "up"        => "up",
        "down"      => "down",
        "C-h"       => "left",
        "C-l"       => "right",
        "left"      => "left",
        "right"     => "right",
        "j"         => "down",
        "k"         => "up",
        "enter"     => "submit",
        "esc"       => "abort",
        "C-g"       => "abort",
        "C-u"       => "clear",
        "space"     => "toggle",
        "C-f"       => "pagedown",
        "C-b"       => "pageup",
        "g g"       => "gotofirst",
        "G"         => "gotolast",
        "q"         => "confirm quit",
        "C-c"       => "quit",
        "x"         => "scrape",
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
        "v"         => "unmark",
        "V"         => "confirm unmark %",
        "C-v"       => "cmd unmark",
        "1"         => "show all",
        "2"         => "show unread",
        "3"         => "show marked",
        "z"         => "zen",
        "/"         => "find",
        "n"         => "searchnext",
        "N"         => "searchprev",
        "="         => "cmd filter ",
        "+ r"       => "filterclear",
        "+ +"       => "filterapply",
        "c w"       => "cmd rename",
        "c d"       => "confirm remove",
        "c x"       => "confirm removeall",
        "c f"       => "cmd feedadd",
        "c a"       => "cmd categoryadd",
        "c u"       => "cmd feedchangeurl",
        "c y"       => "yank",
        "c p"       => "paste after",
        "c P"       => "paste before",
        "c c"       => "cmd tagchangecolor",
        "S c"       => "share clipboard",
        "S r"       => "share reddit",
        "S m"       => "share mastodon",
        "S t"       => "share telegram",
        "S i"       => "share instapaper",
        "?"         => "helpinput",
    ]
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_amount: 10,
            timeout_millis: 5000,
            mappings: generate_default_input_commands(),
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
                self.mappings.shift_remove(&key);
            });

        Ok(())
    }

    pub fn match_single_key(&self, key: &Key) -> Option<&CommandSequence> {
        self.mappings.get(&KeySequence { keys: vec![*key] })
    }

    pub fn match_single_key_to_single_command(&self, key: &Key) -> Option<&Command> {
        self.match_single_key(key).and_then(|command_sequence| {
            let first = command_sequence.commands.first();
            first.filter(|_| command_sequence.commands.len() == 1)
        })
    }
}
