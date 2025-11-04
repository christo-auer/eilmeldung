pub mod article_content;
pub mod articles_list;
pub mod command_input;
pub mod feeds_list;
pub mod tooltip;

use crate::prelude::*;

pub mod prelude {
    pub use super::article_content::ArticleContent;
    pub use super::articles_list::prelude::*;
    pub use super::command_input::CommandInput;
    pub use super::feeds_list::FeedList;
    pub use super::tooltip::{Tooltip, TooltipFlavor, tooltip};
}

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use throbber_widgets_tui::Throbber;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.state == AppState::ArticleContentDistractionFree {
            self.article_content.render(area, buf);
            return;
        }

        let feeds_width = match self.state {
            AppState::FeedSelection => self.config.theme.feeds_list_focus_width_percent,
            _ => 100 - self.config.theme.articles_list_focused_width_percent,
        };

        let articles_width = 100 - feeds_width;

        let [top, middle, command_line, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top: fixed 1 line
                Constraint::Min(0),    // Middle: takes remaining space
                Constraint::Length(if self.command_line.is_active() { 3 } else { 0 }),
                Constraint::Length(1), // Bottom: fixed 1 line
            ])
            .areas(area);

        let [feeds_list_chunk, articles_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages(vec![
                feeds_width,
                articles_width,
            ]))
            .areas(middle);

        let [articles_list_chunk, article_content_chunk] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.config.theme.articles_list_height_lines + 1),
                Constraint::Fill(1),
            ])
            .areas(articles_chunk);

        // render stuff
        self.feed_list.render(feeds_list_chunk, buf);
        self.articles_list.render(articles_list_chunk, buf);
        self.article_content.render(article_content_chunk, buf);

        let use_type = if self.news_flash_utils.is_async_operation_running() {
            throbber_widgets_tui::WhichUse::Spin
        } else {
            throbber_widgets_tui::WhichUse::Full
        };

        let top_line = Throbber::default()
            .label("+++ eilmeldung +++")
            .throbber_style(self.config.theme.statusbar)
            .style(self.config.theme.statusbar)
            .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
            .use_type(use_type);

        StatefulWidget::render(top_line, top, buf, &mut self.async_operation_throbber);

        if self.command_line.is_active() {
            self.command_line.render(command_line, buf);
        }

        let bottom_line = self.tooltip.to_line(&self.config);

        bottom_line.render(bottom, buf);
    }
}
