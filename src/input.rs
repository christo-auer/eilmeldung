use crossterm::event::{KeyCode, KeyEvent};
use log::{debug, info, trace};

use crate::commands::{Command, Message};
use crate::config::InputConfig;
use crate::ui::articles_list::ArticleScope;

// TODO make configurable
// TODO support key sequences
// TODO modifiers
pub fn translate_to_commands(_input_config: &InputConfig, key_event: KeyEvent) -> Vec<Message> {
    use Command::*;
    use KeyCode::*;

    let commands = match key_event.code {
        Char('j') => vec![Message::Command(NavigateDown)],
        Char('k') => vec![Message::Command(NavigateUp)],
        Char('h') => vec![Message::Command(NavigateLeft)],
        Char('l') => vec![Message::Command(NavigateRight)],
        Char('q') => {
            info!("Quit command triggered by user");
            vec![Message::Command(ApplicationQuit)]
        }
        Char('r') => {
            info!("Sync command triggered by user");
            vec![Message::Command(FeedsSync)]
        }
        Char('s') => {
            debug!("Scrape article command triggered");
            vec![Message::Command(ArticleScrape)]
        }
        Char(' ') => vec![Message::Command(PanelFocusNext)],
        Backspace => vec![Message::Command(PanelFocusPrevious)],
        Tab => vec![Message::Command(PanelFocusNextCyclic)],
        BackTab => vec![Message::Command(PanelFocusPreivousCyclic)],
        Char('o') => {
            debug!("Open in browser and mark as read");
            vec![Message::Command(ArticleOpenInBrowser), Message::Command(ArticleSetCurrentAsRead), Message::Command(ArticleListSelectNextUnread)]
        }
        Char('n') => {
            debug!("Mark as read and select next unread");
            vec![Message::Command(ArticleSetCurrentAsRead), Message::Command(ArticleListSelectNextUnread)]
        }
        Char('u') => {
            debug!("Mark current as unread");
            vec![Message::Command(ArticleSetCurrentAsUnread)]
        }
        Char('U') => {
            debug!("Toggle current read status");
            vec![Message::Command(ArticleCurrentToggleRead)]
        }
        Char('a') => {
            info!("Mark all as read");
            vec![Message::Command(ArticleListSetAllRead)]
        }
        Char('A') => {
            info!("Mark all as unread");
            vec![Message::Command(ArticleListSetAllUnread)]
        }
        Char('1') => {
            debug!("Switch to All articles scope");
            vec![Message::Command(ArticleListSetScope(ArticleScope::All))]
        }
        Char('2') => {
            debug!("Switch to Unread articles scope");
            vec![Message::Command(ArticleListSetScope(ArticleScope::Unread))]
        }
        Char('3') => {
            debug!("Switch to Marked articles scope");
            vec![Message::Command(ArticleListSetScope(ArticleScope::Marked))]
        }
        Char('z') => {
            debug!("Switch distraction free mode");
            vec![Message::Command(ToggleDistractionFreeMode)]
        }
        _ => {
            trace!("Unhandled key event: {:?}", key_event);
            vec![]
        }
    };

    if !commands.is_empty() {
        trace!(
            "Key {:?} translated to {} commands",
            key_event.code,
            commands.len()
        );
    }

    commands
}
