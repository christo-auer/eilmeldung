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

fn generate_default_input_commands() -> HashMap<KeySequence, CommandSequence> {
    use Command::*;
    vec![
        ("j".into(), NavigateDown.into()),
        ("C-f".into(), NavigatePageDown.into()),
        ("C-b".into(), NavigatePageUp.into()),
        ("g g".into(), NavigateFirst.into()),
        ("G".into(), NavigateLast.into()),
        ("k".into(), NavigateUp.into()),
        ("h".into(), NavigateLeft.into()),
        ("l".into(), NavigateRight.into()),
        ("q".into(), ApplicationQuit.into()),
        ("r".into(), FeedsSync.into()),
        ("s".into(), ArticleCurrentScrape.into()),
        ("g f".into(), PanelFocus(AppState::FeedSelection).into()),
        ("g a".into(), PanelFocus(AppState::ArticleSelection).into()),
        ("g c".into(), PanelFocus(AppState::ArticleContent).into()),
        (":".into(), CommandLineOpen(None).into()),
        ("space".into(), PanelFocusNext.into()),
        ("backspace".into(), PanelFocusPrevious.into()),
        ("tab".into(), PanelFocusNextCyclic.into()),
        ("backtab".into(), PanelFocusPreviousCyclic.into()),
        (
            "o".into(),
            vec![
                ArticleCurrentOpenInBrowser,
                ArticleCurrentSetRead,
                ArticleListSelectNextUnread,
            ]
            .into(),
        ),
        (
            "J".into(),
            vec![ArticleCurrentSetRead, ArticleListSelectNextUnread].into(),
        ),
        ("u".into(), ArticleCurrentSetUnread.into()),
        ("U".into(), ArticleCurrentToggleRead.into()),
        ("a".into(), ArticleListSetAllRead.into()),
        ("A".into(), ArticleListSetAllUnread.into()),
        ("1".into(), ArticleListSetScope(ArticleScope::All).into()),
        ("2".into(), ArticleListSetScope(ArticleScope::Unread).into()),
        ("3".into(), ArticleListSetScope(ArticleScope::Marked).into()),
        ("z".into(), ToggleDistractionFreeMode.into()),
        ("/".into(), CommandLineOpen(Some("/".into())).into()),
        ("n".into(), ArticleListSearchNext.into()),
        ("N".into(), ArticleListSearchPrevious.into()),
        ("= enter".into(), CommandLineOpen(Some("=".into())).into()),
        ("= r".into(), ArticleListFilterClear.into()),
        ("= =".into(), ArticleListFilterApply.into()),
    ]
    .into_iter()
    .collect()
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
