use std::sync::Arc;

use crate::prelude::*;

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget},
};
use ratatui_textarea::TextArea;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct PopupState<'a> {
    title: String,
    contents: Text<'a>,
    is_modal: bool,
    search_input: Option<TextArea<'a>>,
    scroll_offset_y: u16,
    scroll_offset_x: u16,
    search_input_active: bool,
}

pub struct HelpPopup<'a> {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,
    state: Option<PopupState<'a>>,
}

impl<'a> HelpPopup<'a> {
    pub fn new(config: Arc<Config>, message_sender: UnboundedSender<Message>) -> Self {
        Self {
            config,
            message_sender,
            state: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state.is_some()
    }

    pub fn is_modal(&self) -> Option<bool> {
        self.state.as_ref().map(|state| state.is_modal)
    }

    fn on_key_event(&mut self, key_event: &KeyEvent) -> color_eyre::Result<()> {
        let Some(state) = self.state.as_ref() else {
            return Ok(());
        };

        let command = self
            .config
            .input_config
            .match_single_key_to_single_command(&Key::from(*key_event))
            .cloned();

        match state.search_input_active {
            true => self.on_key_event_search_input(key_event, command)?,
            false => {
                if let Some(command) = command {
                    self.on_key_event_modal(&command)?;
                }
            }
        }

        Ok(())
    }

    fn on_key_event_modal(&mut self, command: &Command) -> color_eyre::Result<()> {
        let Some(state) = self.state.as_mut() else {
            return Ok(());
        };

        use Command as C;

        match command {
            C::NavigateUp => {
                state.scroll_offset_y = (state.scroll_offset_y.saturating_sub(1))
                    .clamp(0, state.contents.height() as u16)
            }
            C::NavigateDown => {
                state.scroll_offset_y = (state.scroll_offset_y.saturating_add(1))
                    .clamp(0, state.contents.height() as u16)
            }
            C::NavigatePageUp => {
                state.scroll_offset_y = (state
                    .scroll_offset_y
                    .saturating_sub(self.config.input_config.scroll_amount as u16))
                .clamp(0, state.contents.height() as u16)
            }
            C::NavigatePageDown => {
                state.scroll_offset_y = (state
                    .scroll_offset_y
                    .saturating_add(self.config.input_config.scroll_amount as u16))
                .clamp(0, state.contents.height() as u16)
            }
            C::NavigateLeft => {
                state.scroll_offset_x = (state.scroll_offset_x.saturating_sub(1))
                    .clamp(0, state.contents.width() as u16)
            }
            C::NavigateRight => {
                state.scroll_offset_x = (state.scroll_offset_x.saturating_add(1))
                    .clamp(0, state.contents.width() as u16)
            }
            C::InputSearch => {
                if state.search_input.is_none() {
                    let mut text_area = TextArea::default();
                    text_area.set_placeholder_text("search term");
                    text_area.set_style(self.config.theme.command_input());
                    state.search_input = Some(text_area);
                }
                state.search_input_active = true;
            }
            C::InputSubmit | C::InputAbort => {
                self.state = None;
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
        let Some(state) = self.state.as_mut() else {
            return Ok(());
        };
        let Some(text_area) = state.search_input.as_mut() else {
            return Ok(());
        };

        match command {
            Some(Command::InputSubmit) => state.search_input_active = false,
            Some(Command::InputAbort) => {
                state.search_input_active = false;
                state.search_input = None;
            }

            Some(Command::InputClear) => {
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
        if let Some(PopupState {
            title,
            contents,
            is_modal,
            scroll_offset_y,
            scroll_offset_x,
            search_input,
            search_input_active,
        }) = self.state.as_ref()
        {
            let (width, height) = (
                (contents.width() + 4).min((area.width as usize).saturating_sub(4)),
                contents.height() + 2,
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

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.config.theme.border_focused())
                .title_top(Line::styled(
                    format!(" {title} "),
                    self.config.theme.header(),
                ))
                .padding(Padding::horizontal(1));

            let inner_area = block.inner(popup_area);

            Widget::render(Clear, popup_area, buf);
            block.render(popup_area, buf);

            if *is_modal {
                match search_input {
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
                        let lines = contents
                            .lines
                            .iter()
                            .filter(|line| {
                                line.spans.iter().any(|span| {
                                    matcher
                                        .fuzzy_match(
                                            span.content.as_ref(),
                                            &search_input.lines()[0],
                                        )
                                        .is_some()
                                })
                            })
                            .cloned()
                            .collect::<Vec<Line>>();

                        let entries: u16 = lines.len() as u16;
                        let paragraph = Paragraph::new(lines).scroll((
                            (*scroll_offset_y).min(entries.saturating_sub(contents_chunk.height)),
                            *scroll_offset_x,
                        ));
                        paragraph.render(contents_chunk, buf);

                        if *search_input_active {
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
                        let paragraph = Paragraph::new(contents.to_owned())
                            .scroll((*scroll_offset_y, *scroll_offset_x));

                        paragraph.render(inner_area, buf);
                    }
                }
            } else {
                contents.render(inner_area, buf);
            }
        }
    }
}

impl<'a> MessageReceiver for HelpPopup<'a> {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut redraw_required = false;
        if let Message::Event(event) = message {
            use Event as E;
            match event {
                E::ShowHelpPopup(title, contents) => {
                    self.state = Some(PopupState {
                        contents: contents.to_owned(),
                        title: title.to_owned(),
                        is_modal: false,
                        search_input: None,
                        scroll_offset_y: 0,
                        scroll_offset_x: 0,
                        search_input_active: false,
                    });
                    redraw_required = true;
                }
                E::ShowModalHelpPopup(title, contents) => {
                    self.state = Some(PopupState {
                        contents: contents.to_owned(),
                        title: title.to_owned(),
                        is_modal: true,
                        search_input: None,
                        scroll_offset_y: 0,
                        scroll_offset_x: 0,
                        search_input_active: false,
                    });
                    redraw_required = true;
                }
                E::HideHelpPopup => {
                    self.state = None;
                    redraw_required = true;
                }
                E::Key(key_event) if self.is_visible() => {
                    self.on_key_event(key_event)?;
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
