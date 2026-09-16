use std::sync::Arc;

use crate::prelude::*;

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    crossterm::event::{Event as TermEvent, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
};
use ratatui_textarea::TextArea;
use tokio::sync::mpsc::UnboundedSender;

pub struct HelpPopup<'a> {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,
    title: String,
    contents: Text<'a>,
    is_modal: bool,
    search_input: Option<TextArea<'a>>,
    scroll_offset_y: u16,
    scroll_offset_x: u16,
    search_input_active: bool,
}

impl<'a> HelpPopup<'a> {
    pub fn new(
        config: Arc<Config>,
        message_sender: UnboundedSender<Message>,
        title: String,
        contents: Text<'a>,
        is_modal: bool,
    ) -> Self {
        Self {
            config,
            message_sender,
            contents,
            title,
            is_modal,
            search_input: None,
            scroll_offset_y: 0,
            scroll_offset_x: 0,
            search_input_active: false,
        }
    }

    fn on_key_event(&mut self, key_event: &KeyEvent) -> color_eyre::Result<TermEventForwarding> {
        let command = self
            .config
            .input_config
            .match_single_key_to_single_command(&Key::from(*key_event))
            .cloned();

        match self.search_input_active {
            true => self.on_key_event_search_input(key_event, command)?,
            false if key_event.is_press() => {
                if let Some(command) = command {
                    self.on_key_event_modal(&command)?;
                }
            }
            _ => {}
        }

        Ok(TermEventForwarding::Consumed)
    }

    fn on_key_event_modal(&mut self, command: &Command) -> color_eyre::Result<()> {
        use Command as C;

        match command {
            C::NavigateUp => {
                self.scroll_offset_y =
                    (self.scroll_offset_y.saturating_sub(1)).clamp(0, self.contents.height() as u16)
            }
            C::NavigateDown => {
                self.scroll_offset_y =
                    (self.scroll_offset_y.saturating_add(1)).clamp(0, self.contents.height() as u16)
            }
            C::NavigatePageUp => {
                self.scroll_offset_y = (self
                    .scroll_offset_y
                    .saturating_sub(self.config.input_config.scroll_amount as u16))
                .clamp(0, self.contents.height() as u16)
            }
            C::NavigatePageDown => {
                self.scroll_offset_y = (self
                    .scroll_offset_y
                    .saturating_add(self.config.input_config.scroll_amount as u16))
                .clamp(0, self.contents.height() as u16)
            }
            C::NavigateLeft => {
                self.scroll_offset_x =
                    (self.scroll_offset_x.saturating_sub(1)).clamp(0, self.contents.width() as u16)
            }
            C::NavigateRight => {
                self.scroll_offset_x =
                    (self.scroll_offset_x.saturating_add(1)).clamp(0, self.contents.width() as u16)
            }
            C::InputSearch => {
                if self.search_input.is_none() {
                    let mut text_area = TextArea::default();
                    text_area.set_placeholder_text("search term");
                    text_area.set_style(self.config.theme.command_input());
                    self.search_input = Some(text_area);
                }
                self.search_input_active = true;
            }
            C::InputSubmit | C::InputAbort => {
                self.message_sender
                    .send(Message::Event(Event::Popup(PopupEvent::HideHelp)))?;
            }
            _ => {}
        }

        Ok(())
    }

    fn on_key_event_search_input(
        &mut self,
        key_event: &KeyEvent,
        command: Option<Command>,
    ) -> color_eyre::Result<()> {
        let Some(text_area) = self.search_input.as_mut() else {
            return Ok(());
        };

        match command {
            Some(Command::InputSubmit) if key_event.is_press() => self.search_input_active = false,
            Some(Command::InputAbort) if key_event.is_press() => {
                self.search_input_active = false;
                self.search_input = None;
            }

            Some(Command::InputClear) if key_event.is_press() => {
                text_area.select_all();
                text_area.delete_char();
            }

            _ => {
                // ignore up/down navigation in text area
                if !matches!(
                    key_event.code,
                    KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::Enter
                        | KeyCode::PageDown
                        | KeyCode::PageUp
                ) {
                    text_area.input(*key_event);
                }
            }
        }

        Ok(())
    }
}

impl<'a> Widget for &HelpPopup<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let (width, height) = (
            (self.contents.width() + 4).min((area.width as usize).saturating_sub(4)),
            self.contents.height() + 2,
        );

        let [popup_area] = Layout::horizontal([Constraint::Length(width as u16)])
            .flex(Flex::Center)
            .areas::<1>(area);

        let [_, popup_area, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(height as u16),
            Constraint::Length(6),
        ])
        .flex(Flex::End)
        .areas::<3>(popup_area);

        let mut block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(*self.config.theme.color_palette().background()))
            .border_type(self.config.border_theme.focused)
            .border_style(self.config.theme.border_focused())
            .title_top(Line::styled(
                format!(" {} ", self.title),
                self.config.theme.header(),
            ))
            .padding(Padding::horizontal(1));

        if self.config.shadows {
            block = block.shadow(Shadow::light_shade());
        }

        let inner_area = block.inner(popup_area);

        Widget::render(Clear, popup_area, buf);
        block.render(popup_area, buf);

        if self.is_modal {
            match self.search_input.as_ref() {
                Some(search_input) => {
                    let [contents_chunk, search_chunk] = Layout::default()
                        .direction(Direction::Vertical)
                        .flex(Flex::End)
                        .constraints(vec![
                            Constraint::Length(height.saturating_sub(1) as u16),
                            Constraint::Length(1),
                        ])
                        .areas(inner_area);

                    let matcher = SkimMatcherV2::default();
                    let lines = self
                        .contents
                        .lines
                        .iter()
                        .filter(|line| {
                            line.spans.iter().any(|span| {
                                matcher
                                    .fuzzy_match(span.content.as_ref(), &search_input.lines()[0])
                                    .is_some()
                            })
                        })
                        .cloned()
                        .collect::<Vec<Line>>();

                    let entries: u16 = lines.len() as u16;
                    let paragraph = Paragraph::new(lines).scroll((
                        (self.scroll_offset_y).min(entries.saturating_sub(contents_chunk.height)),
                        self.scroll_offset_x,
                    ));
                    paragraph.render(contents_chunk, buf);

                    if self.search_input_active {
                        search_input.render(search_chunk, buf);
                    } else {
                        Span::styled(
                            search_input.lines()[0].as_str(),
                            self.config.theme.command_input(),
                        )
                        .render(search_chunk, buf);
                    }
                }
                None => {
                    let paragraph = Paragraph::new(self.contents.to_owned())
                        .scroll((self.scroll_offset_y, self.scroll_offset_x));

                    paragraph.render(inner_area, buf);
                }
            }
        } else {
            (&self.contents).render(inner_area, buf);
        }
    }
}

impl TermEventHandler for HelpPopup<'_> {
    async fn process_term_event(
        &mut self,
        event: &TermEvent,
    ) -> color_eyre::Result<TermEventForwarding> {
        if let TermEvent::Key(key_event) = event
            && self.is_modal
        {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
            self.on_key_event(key_event)
        } else {
            Ok(TermEventForwarding::PassOn)
        }
    }
}

impl<'a> MessageReceiver for HelpPopup<'a> {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut redraw_required = false;
        if let Message::Event(event) = message {
            use Event as E;
            match event {
                E::ConfigReloaded(config) => {
                    self.config = Arc::clone(config);
                    redraw_required = true;
                }
                _ => {}
            }
        }

        if redraw_required {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
