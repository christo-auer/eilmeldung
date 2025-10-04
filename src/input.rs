use crossterm::event::{KeyCode, KeyEvent, ModifierKeyCode};

use crate::commands::Command;
use crate::config::InputConfig;

// TODO make configurable
// TODO support key sequences
// TODO modifiers
pub fn translate_to_command(input_config: &InputConfig, key_event: KeyEvent) -> Option<Command> {
    use Command::*;
    use KeyCode::*;

    match key_event.code {
        Char('j') => Some(NavigateDown),
        Char('k') => Some(NavigateUp),
        Char('h') => Some(NavigateLeft),
        Char('l') => Some(NavigateRight),
        Enter => Some(SelectionSubmit),
        Char('q') => Some(ApplicationQuit),
        _ => None,
    }
}
