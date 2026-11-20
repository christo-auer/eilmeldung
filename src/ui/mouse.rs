use std::{sync::Arc, time::Duration};

use crate::prelude::*;

use ratatui::crossterm::event::{MouseButton, MouseEventKind};

use ratatui::crossterm::event::Event as TermEvent;
use tokio::{sync::mpsc::UnboundedSender, time::Instant};

#[derive(getset::MutGetters)]
pub struct MouseInputHandler {
    message_sender: UnboundedSender<Message>,
    config: Arc<Config>,
    state: AppState,
    panel_areas: PanelAreas,

    active_drag_resize: Option<Panel>,

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
            active_drag_resize: None,
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
            MouseEventKind::Down(MouseButton::Left) if self.active_drag_resize.is_none() => {
                if matches!(self.state, AppState::ArticleContentDistractionFree) {
                    return Ok(None);
                }

                if self.panel_areas.is_on_article_list_border(col, row) {
                    log::trace!("start dragging border article list");
                    self.active_drag_resize = Some(Panel::ArticleList);
                    Ok(Some(TermEventForwarding::Consumed))
                } else if self.panel_areas.is_on_feed_list_border(col, row) {
                    log::trace!("start dragging border feed list");
                    self.active_drag_resize = Some(Panel::FeedList);
                    Ok(Some(TermEventForwarding::Consumed))
                } else {
                    Ok(None)
                }
            }
            MouseEventKind::Drag(MouseButton::Left)
                if matches!(self.active_drag_resize, Some(Panel::ArticleList)) =>
            {
                // Calculate the new articles list height based on drag position
                let articles_top = self.panel_areas.articles_list().y;
                let new_articles_height = row.saturating_sub(articles_top);

                // only redraw if height has changed
                if self.panel_areas.articles_list.height != new_articles_height {
                    log::trace!("dragging border of article list: {new_articles_height}");
                    self.message_sender.send(Message::Command(Command::In(
                        Panel::ArticleList,
                        Box::new(Command::PanelResize(ResizeArg::Absolute(
                            Dimension::Length(new_articles_height),
                        ))),
                    )))?;
                }

                Ok(Some(TermEventForwarding::Consumed))
            }

            MouseEventKind::Drag(MouseButton::Left)
                if matches!(self.active_drag_resize, Some(Panel::FeedList)) =>
            {
                // Calculate the new articles list height based on drag position
                let feeds_left = self.panel_areas.feed_list.x;
                let new_feeds_width = col.saturating_sub(feeds_left);

                // only redraw if height has changed
                if self.panel_areas.feed_list.width != new_feeds_width {
                    log::trace!("dragging border of feeds: {new_feeds_width}");
                    self.message_sender.send(Message::Command(Command::In(
                        Panel::FeedList,
                        Box::new(Command::PanelResize(ResizeArg::Absolute(
                            Dimension::Length(new_feeds_width),
                        ))),
                    )))?;
                }

                Ok(Some(TermEventForwarding::Consumed))
            }

            MouseEventKind::Up(MouseButton::Left) if self.active_drag_resize.is_some() => {
                log::trace!("dragging stopped");
                self.active_drag_resize.take();
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
        match message {
            Message::Event(Event::PanelAreasChanged(panel_areas)) => {
                self.panel_areas = *panel_areas
            }
            Message::Event(Event::ApplicationStateChanged(state)) => self.state = *state,
            _ => {}
        }

        Ok(())
    }
}
