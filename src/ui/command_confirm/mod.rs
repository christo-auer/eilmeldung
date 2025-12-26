use std::sync::Arc;

use ratatui::{
    crossterm::event::KeyCode,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Widget},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

#[derive(getset::CopyGetters)]
pub struct CommandConfirm {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,

    command_to_confirm: Option<Command>,

    #[getset(get_copy = "pub")]
    is_active: bool,
}

impl CommandConfirm {
    pub fn new(config: Arc<Config>, message_sender: UnboundedSender<Message>) -> Self {
        Self {
            config,
            message_sender,
            command_to_confirm: None,
            is_active: false,
        }
    }
}

impl Widget for &CommandConfirm {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_active {
                self.config.theme.border_focused()
            } else {
                self.config.theme.border()
            })
            .padding(Padding::horizontal(1))
            .border_type(BorderType::Thick);

        let inner = block.inner(area);

        if let Some(command) = self.command_to_confirm.as_ref() {
            let prompt = Line::from(vec![
                Span::from(format!(" {command}? ")).style(self.config.theme.paragraph()),
                Span::from("(y/n)").style(self.config.theme.header()),
            ]);
            prompt.render(inner, buf);
        }
        block.render(area, buf);
    }
}

impl MessageReceiver for CommandConfirm {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut needs_redraw = false;
        if let Message::Command(Command::CommandConfirm(command)) = message {
            self.is_active = true;
            self.command_to_confirm = Some(command.as_ref().clone());
            needs_redraw = true;
        }

        if let Message::Event(Event::Key(key_event)) = message {
            match key_event.code {
                KeyCode::Char('y') if self.is_active => {
                    self.is_active = false;
                    self.message_sender
                        .send(Message::Command(self.command_to_confirm.take().unwrap()))?;
                    needs_redraw = true;
                }

                KeyCode::Char('n') | KeyCode::Esc if self.is_active => {
                    self.is_active = false;
                    self.command_to_confirm = None;
                    needs_redraw = true;
                }

                _ => {}
            }
        }

        if needs_redraw {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
