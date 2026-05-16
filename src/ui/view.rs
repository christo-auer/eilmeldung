use crate::prelude::*;

use ratatui::{layout::Flex, prelude::*};
use throbber_widgets_tui::Throbber;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // render status bar
        let area = self.render_status_bar(area, buf);

        // render command line (if visible)
        let (panels_chunk, command_line_chunk) = self.compute_command_line_chunk(area.to_owned());

        // render the main panels
        self.render_panels(panels_chunk, buf);

        self.render_command_line(buf, command_line_chunk);

        // and finally the pop if visible
        if self.help_popup.is_visible() {
            self.help_popup.render(area, buf);
        }
    }
}

impl App {
    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) -> Rect {
        // in zen mode we only lower status bar if the tooltip is Warning or Error
        let render_bottom_bar = !matches!(self.state, AppState::ArticleContentDistractionFree)
            || matches!(
                self.tooltip.flavor,
                TooltipFlavor::Warning | TooltipFlavor::Error
            );

        let [middle, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Middle: takes remaining space
                Constraint::Length(if render_bottom_bar { 1 } else { 0 }), // Bottom: fixed 1 line
            ])
            .areas(area);

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

        if render_bottom_bar {
            // fill top line with status bar color
            Block::default()
                .style(self.config.theme.statusbar())
                .render(bottom, buf);

            let [bottom_left, bottom_main, status, bottom_right] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::Center)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(bottom.width.saturating_sub(3)),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .areas::<4>(bottom);

            let tooltip_line = self.tooltip.to_line(&self.config);

            Span::styled(status_span.content, tooltip_line.style).render(status, buf);
            Span::styled("", tooltip_line.style.not_reversed()).render(bottom_left, buf);
            Span::styled("", tooltip_line.style.not_reversed()).render(bottom_right, buf);
            tooltip_line.render(bottom_main, buf);
        }

        middle
    }

    fn render_command_line(&mut self, buf: &mut Buffer, command_line_chunk: Rect) {
        if self.command_input.is_active() {
            self.command_input.render(command_line_chunk, buf);
        } else if self.command_confirm.is_active() {
            self.command_confirm.render(command_line_chunk, buf);
        }
    }

    fn compute_command_line_chunk(&mut self, area: Rect) -> (Rect, Rect) {
        if self.command_input.is_active() || self.command_confirm.is_active() {
            let [panels_chunk, command_line_chunk] =
                Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).areas::<2>(area);

            (panels_chunk, command_line_chunk)
        } else {
            (area, Default::default())
        }
    }

    fn render_panels(&mut self, area: Rect, buf: &mut Buffer) {
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

        let (articles_constraint_height, article_content_constraint_height) =
            if let Some(override_height) = self.articles_height_override {
                // User is dragging the border — use absolute heights
                (Constraint::Length(override_height), Constraint::Min(0))
            } else {
                match self.state {
                    AppState::FeedSelection | AppState::ArticleSelection => (
                        self.config.article_list_focused_height.as_constraint(),
                        self.config
                            .article_list_focused_height
                            .as_complementary_constraint(area.height),
                    ),
                    _ => (
                        self.config
                            .article_content_focused_height
                            .as_complementary_constraint(area.height),
                        self.config.article_content_focused_height.as_constraint(),
                    ),
                }
            };

        let [feeds_list_chunk, articles_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![feeds_constraint_width, articles_constraint_width])
            .areas::<2>(area);

        let [articles_list_chunk, article_content_chunk] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                articles_constraint_height,
                article_content_constraint_height,
            ])
            .areas(articles_chunk);

        // store areas for mouse hit-testing
        *self.panel_areas.feed_list_mut() = feeds_list_chunk;
        *self.panel_areas.articles_list_mut() = articles_list_chunk;
        *self.panel_areas.article_content_mut() = article_content_chunk;

        if !self.feed_list.is_focused() {
            self.feed_list.render(feeds_list_chunk, buf);
        }
        if !self.articles_list.is_focused() {
            self.articles_list.render(articles_list_chunk, buf);
        }
        if !self.article_content.is_focused() {
            self.article_content.render(article_content_chunk, buf);
        }

        // render the focused panel last so that its border drawn over the other borders
        if self.feed_list.is_focused() {
            self.feed_list.render(feeds_list_chunk, buf);
        } else if self.articles_list.is_focused() {
            self.articles_list.render(articles_list_chunk, buf);
        } else if self.article_content.is_focused() {
            self.article_content.render(article_content_chunk, buf);
        }
    }
}
