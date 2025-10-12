use crossterm::event::{KeyCode, KeyEvent};

use crate::commands::Command;
use crate::config::InputConfig;
use crate::ui::articles_list::ArticleScope;

// TODO make configurable
// TODO support key sequences
// TODO modifiers
pub fn translate_to_commands(_input_config: &InputConfig, key_event: KeyEvent) -> Vec<Command> {
    use Command::*;
    use KeyCode::*;

    match key_event.code {
        Char('j') => vec![NavigateDown],
        Char('k') => vec![NavigateUp],
        Char('h') => vec![NavigateLeft],
        Char('l') => vec![NavigateRight],
        Char('q') => vec![ApplicationQuit],
        Char('r') => vec![Sync],
        Char(' ') => vec![FocusNext],
        Backspace => vec![FocusPrevious],
        Tab => vec![CyclicFocusNext],
        BackTab => vec![CyclicFocusPrevious],
        Char('o') => vec![OpenInBrowser, SetCurrentAsRead, SelectNextUnread],
        Char('n') => vec![SetCurrentAsRead, SelectNextUnread],
        Char('u') => vec![SetCurrentAsUnread],
        Char('U') => vec![ToggleCurrentRead],
        Char('a') => vec![SetAllRead],
        Char('A') => vec![SetAllUnread],
        Char('1') => vec![SetArticleScope(ArticleScope::All)],
        Char('2') => vec![SetArticleScope(ArticleScope::Unread)],
        Char('3') => vec![SetArticleScope(ArticleScope::Marked)],
        _ => vec![],
    }
}
