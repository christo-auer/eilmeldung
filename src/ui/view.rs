use crate::prelude::*;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Widget,
};
use throbber_widgets_tui::Throbber;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.state == AppState::ArticleContentDistractionFree {
            self.article_content.render(area, buf);
            return;
        }

        let (feeds_constraint_width, articles_constraint_width) = match self.state {
            AppState::FeedSelection => (
                self.config.feed_list_focused_width.as_constraint(),
                self.config
                    .feed_list_focused_width
                    .as_complementary_constraint(area.width),
            ),
            _ => (
                self.config
                    .article_list_focused_width
                    .as_complementary_constraint(area.width),
                self.config.article_list_focused_width.as_constraint(),
            ),
        };

        let command_line_visible =
            self.command_input.is_active() || self.command_confirm.is_active();

        let [top, middle, command_line, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top: fixed 1 line
                Constraint::Min(0),    // Middle: takes remaining space
                Constraint::Length(if command_line_visible { 3 } else { 0 }),
                Constraint::Length(1), // Bottom: fixed 1 line
            ])
            .areas(area);

        let (articles_constraint_height, article_content_constraint_height) = match self.state {
            AppState::FeedSelection | AppState::ArticleSelection => (
                self.config.article_list_focused_height.as_constraint(),
                self.config
                    .article_list_focused_height
                    .as_complementary_constraint(middle.height),
            ),
            _ => (
                self.config
                    .article_content_focused_height
                    .as_complementary_constraint(middle.height),
                self.config.article_content_focused_height.as_constraint(),
            ),
        };

        let [feeds_list_chunk, articles_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![feeds_constraint_width, articles_constraint_width])
            .areas::<2>(middle);

        let [articles_list_chunk, article_content_chunk] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                articles_constraint_height,
                article_content_constraint_height,
            ])
            .areas(articles_chunk);

        // render stuff
        self.feed_list.render(feeds_list_chunk, buf);
        self.articles_list.render(articles_list_chunk, buf);
        self.article_content.render(article_content_chunk, buf);

        let status_span = if self.is_offline {
            // when offline display offline icon
            Span::styled(
                format!("{} ", self.config.offline_icon),
                self.config.theme.statusbar(),
            )
        } else {
            // when online display throbber
            let use_type = if self.news_flash_utils.is_async_operation_running() {
                throbber_widgets_tui::WhichUse::Spin
            } else {
                throbber_widgets_tui::WhichUse::Empty
            };
            Throbber::default()
                .throbber_style(self.config.theme.statusbar())
                .style(self.config.theme.statusbar())
                .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
                .use_type(use_type)
                .to_symbol_span(&self.async_operation_throbber)
        };

        let eilmeldung_span = Span::styled(" +++ eilmeldung +++ ", self.config.theme.statusbar());

        let title = Line::from(vec![eilmeldung_span, status_span.clone()])
            .style(self.config.theme.statusbar());

        title.render(top, buf);

        if self.command_input.is_active() {
            self.command_input.render(command_line, buf);
        } else if self.command_confirm.is_active() {
            self.command_confirm.render(command_line, buf);
        }

        if self.help_popup.is_visible() {
            self.help_popup.render(area, buf);
        }

        let bottom_line = self.tooltip.to_line(&self.config);

        bottom_line.render(bottom, buf);
    }
}
