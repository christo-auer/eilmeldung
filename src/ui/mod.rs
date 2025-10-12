pub mod article_content;
pub mod articles_list;
pub mod feeds_list;
pub mod tooltip;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use throbber_widgets_tui::{Throbber, symbols::throbber};

use crate::app::{App, AppState};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let feeds_width = match self.state {
            AppState::FeedSelection => self.config.theme.feeds_focused_width,
            _ => 100 - self.config.theme.articles_focused_width,
        };

        let article_content_height = match self.state {
            AppState::ArticleContent => self.config.theme.article_content_focused_height,
            _ => 100 - self.config.theme.article_content_focused_height,
        };

        let articles_width = 100 - feeds_width;
        let article_list_height = 100 - article_content_height;

        let [top, middle, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top: fixed 1 line
                Constraint::Min(0),    // Middle: takes remaining space
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
            .constraints(Constraint::from_percentages(vec![
                article_list_height,
                article_content_height,
            ]))
            .areas(articles_chunk);

        // render stuff
        self.feed_list.render(feeds_list_chunk, buf);
        self.articles_list.render(articles_list_chunk, buf);
        self.article_content.render(article_content_chunk, buf);

        let use_type = if self.news_flash_async_manager.is_async_operation_running() {
            throbber_widgets_tui::WhichUse::Spin
        } else {
            throbber_widgets_tui::WhichUse::Full
        };

        let top_line = Throbber::default()
            .label("eilmeldung")
            .throbber_style(self.config.theme.statusbar)
            .style(self.config.theme.statusbar)
            .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
            .use_type(use_type);

        StatefulWidget::render(top_line, top, buf, &mut self.async_operation_throbber);

        let bottom_line = self.tooltip.to_text(&self.config);

        bottom_line.render(bottom, buf);
    }
}
