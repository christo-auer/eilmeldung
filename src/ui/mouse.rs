use crate::prelude::*;

use getset::{Getters, MutGetters};
use ratatui::{
    crossterm::event::{MouseButton, MouseEventKind},
    prelude::Rect,
};

/// Stores the last rendered areas of the three main panels for mouse hit-testing.
#[derive(Default, Clone, Copy, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PanelAreas {
    feed_list: Rect,
    articles_list: Rect,
    article_content: Rect,
}

impl PanelAreas {
    pub(super) fn panel_at(&self, col: u16, row: u16) -> Option<Panel> {
        if self.feed_list.contains((col, row).into()) {
            Some(Panel::FeedList)
        } else if self.articles_list.contains((col, row).into()) {
            Some(Panel::ArticleList)
        } else if self.article_content.contains((col, row).into()) {
            Some(Panel::ArticleContent)
        } else {
            None
        }
    }

    /// Returns the row offset relative to the inner area of the articles panel (excluding border).
    pub(super) fn article_row_offset(&self, row: u16) -> Option<u16> {
        let area = self.articles_list;
        // Account for the border (1 row top)
        let inner_top = area.y + 1;
        let inner_bottom = area.y + area.height.saturating_sub(1);
        if row >= inner_top && row < inner_bottom {
            Some(row - inner_top)
        } else {
            None
        }
    }

    /// Returns true if the row is on the horizontal border between the articles list and article content.
    pub(super) fn is_on_horizontal_border(&self, col: u16, row: u16) -> bool {
        // The border is at the bottom edge of articles_list / top edge of article_content
        let border_row = self.articles_list.y + self.articles_list.height;
        let in_column_range =
            col >= self.articles_list.x && col < self.articles_list.x + self.articles_list.width;
        (row == border_row || row == border_row.saturating_sub(1)) && in_column_range
    }
}

impl App {
    pub(super) fn handle_mouse_event(
        &mut self,
        mouse_event: &ratatui::crossterm::event::MouseEvent,
    ) -> color_eyre::Result<()> {
        // Skip mouse events when a modal/dialog is active
        if self.command_input.is_active()
            || self.command_confirm.is_active()
            || self.help_popup.is_modal().unwrap_or(false)
        {
            return Ok(());
        }

        let col = mouse_event.column;
        let row = mouse_event.row;

        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if clicking on the horizontal border to start a drag-resize
                if self.panel_areas.is_on_horizontal_border(col, row) {
                    self.drag_resize_active = true;
                    return Ok(());
                }

                if let Some(panel) = self.panel_areas.panel_at(col, row) {
                    // Focus the clicked panel (only if not in distraction free mode)
                    let target_state: AppState = panel.into();
                    if !matches!(self.state, AppState::ArticleContentDistractionFree)
                        && self.state != target_state
                    {
                        self.switch_state(target_state)?;
                    }

                    match panel {
                        Panel::ArticleList => {
                            if let Some(row_offset) = self.panel_areas.article_row_offset(row) {
                                self.message_sender
                                    .send(Message::Event(Event::MouseArticleClick(row_offset)))?;
                            }
                        }
                        Panel::FeedList => {
                            self.message_sender
                                .send(Message::Event(Event::MouseFeedClick(col, row)))?;
                        }
                        _ => {}
                    }

                    self.message_sender
                        .send(Message::Command(Command::Redraw))?;
                }
            }

            MouseEventKind::Drag(MouseButton::Left) if self.drag_resize_active => {
                // Calculate the new articles list height based on drag position
                let articles_top = self.panel_areas.articles_list().y;
                let content_bottom = self.panel_areas.article_content().y
                    + self.panel_areas.article_content().height;
                let total_height = content_bottom.saturating_sub(articles_top);
                // Clamp: minimum 3 rows for each panel
                let new_articles_height = row
                    .saturating_sub(articles_top)
                    .clamp(3, total_height.saturating_sub(3));

                let old_articles_height =
                    self.articles_height_override.replace(new_articles_height);

                // only redraw if height has changed
                if let Some(old_articles_height) = old_articles_height
                    && old_articles_height != new_articles_height
                {
                    self.message_sender
                        .send(Message::Command(Command::Redraw))?;
                }
            }

            MouseEventKind::Up(MouseButton::Left) => {
                self.drag_resize_active = false;
            }

            MouseEventKind::ScrollDown => {
                if let Some(panel) = self.panel_areas.panel_at(col, row) {
                    self.message_sender
                        .send(Message::Event(Event::MouseScrollDown(panel)))?;
                    self.message_sender
                        .send(Message::Command(Command::Redraw))?;
                }
            }

            MouseEventKind::ScrollUp => {
                if let Some(panel) = self.panel_areas.panel_at(col, row) {
                    self.message_sender
                        .send(Message::Event(Event::MouseScrollUp(panel)))?;
                    self.message_sender
                        .send(Message::Command(Command::Redraw))?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
