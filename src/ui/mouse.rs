use std::{sync::Arc, time::Duration};

use crate::prelude::*;

use getset::{Getters, MutGetters};
use ratatui::{
    crossterm::event::{MouseButton, MouseEventKind},
    prelude::Rect,
};

use ratatui::crossterm::event::Event as TermEvent;
use tokio::{sync::mpsc::UnboundedSender, time::Instant};

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
        if row >= inner_top && row <= inner_bottom {
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
        row == border_row && in_column_range
    }
}

#[derive(getset::MutGetters, getset::Getters)]
pub struct MouseInputHandler {
    message_sender: UnboundedSender<Message>,
    config: Arc<Config>,
    #[getset(get_mut = "pub")]
    panel_areas: PanelAreas,

    drag_resize_active: bool,

    #[getset(get = "pub")]
    articles_height_override: Option<u16>,

    state: AppState,

    #[getset(get_mut = "pub")]
    enabled: bool,

    last_scroll_event: Instant,
}

impl MouseInputHandler {
    pub fn new(config: Arc<Config>, message_sender: UnboundedSender<Message>) -> Self {
        Self {
            enabled: true,
            message_sender,
            config,
            panel_areas: Default::default(),
            drag_resize_active: false,
            articles_height_override: None,
            state: Default::default(),
            last_scroll_event: Instant::now(),
        }
    }

    fn handle_content_resize_dragging(
        &mut self,
        col: u16,
        row: u16,
        mouse_event: &ratatui::crossterm::event::MouseEvent,
    ) -> color_eyre::Result<Option<TermEventForwarding>> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left)
                if !matches!(self.state, AppState::ArticleContentDistractionFree)
                    && self.panel_areas.is_on_horizontal_border(col, row) =>
            {
                log::trace!("start dragging border");
                self.drag_resize_active = true;
                Ok(Some(TermEventForwarding::Consumed))
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
                    log::trace!("dragging border: {new_articles_height}");
                    self.message_sender
                        .send(Message::Command(Command::Redraw))?;
                }

                Ok(Some(TermEventForwarding::Consumed))
            }
            MouseEventKind::Up(MouseButton::Left) if self.drag_resize_active => {
                log::trace!("dragging stopped");
                self.drag_resize_active = false;
                Ok(Some(TermEventForwarding::Consumed))
            }

            _ => Ok(None),
        }
    }
}

impl TermEventHandler for MouseInputHandler {
    async fn process_term_event(
        &mut self,
        event: &TermEvent,
    ) -> color_eyre::Result<TermEventForwarding> {
        if !self.enabled {
            return Ok(TermEventForwarding::PassOn);
        }

        let TermEvent::Mouse(mouse_event) = event else {
            return Ok(TermEventForwarding::PassOn);
        };

        let col = mouse_event.column;
        let row = mouse_event.row;

        let TermEvent::Mouse(mouse_event) = event else {
            return Ok(TermEventForwarding::PassOn);
        };

        if self.config.mouse_config.content_resize
            && let Some(event_forwarding) =
                self.handle_content_resize_dragging(col, row, mouse_event)?
        {
            return Ok(event_forwarding);
        }

        let mouse_input = MouseInput::from(*mouse_event);

        if mouse_input.is_scroll_event()
            && Instant::now().duration_since(self.last_scroll_event)
                < Duration::from_millis(self.config.mouse_config.scroll_debounce_millis)
        {
            return Ok(TermEventForwarding::Consumed);
        } else {
            self.last_scroll_event = Instant::now();
        }

        let select = mouse_input
            .kind()
            .map(|kind| matches!(kind, MouseEventKind::Down(..)))
            .unwrap_or(false);

        if let Some(panel) = self.panel_areas.panel_at(col, row) {
            // Focus the clicked panel (only if not in distraction free mode)
            // let target_state: AppState = panel.into();

            if select {
                match panel {
                    Panel::ArticleList => {
                        if let Some(row_offset) = self.panel_areas.article_row_offset(row) {
                            self.message_sender
                                .send(Message::Event(Event::MouseArticleSelect(row_offset)))?;
                        }
                    }
                    Panel::FeedList => {
                        self.message_sender
                            .send(Message::Event(Event::MouseFeedSelect(col, row)))?;
                    }
                    _ => {}
                }
            }

            self.message_sender
                .send(Message::Command(Command::Redraw))?;

            if let Some(command_sequence) = self.config.mouse_config.get_mapping(mouse_input, panel)
            {
                self.message_sender
                    .send(Message::Batch(command_sequence.commands.to_owned()))?;
                return Ok(TermEventForwarding::Consumed);
            }
        }

        Ok(if select {
            TermEventForwarding::Consumed
        } else {
            TermEventForwarding::PassOn
        })
    }
}

impl MessageReceiver for MouseInputHandler {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Event(Event::ApplicationStateChanged(new_state)) = message {
            self.state = *new_state;
        }

        Ok(())
    }
}
