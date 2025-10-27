use crate::prelude::*;

use std::{str::FromStr, sync::Arc};

use log::info;
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Block, BorderType, Borders},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

pub struct CommandInput {
    config: Arc<Config>,

    _news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    preset_command: Option<String>,
    text_input: TextArea<'static>,

    is_active: bool,
}

impl CommandInput {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            _news_flash_utils: news_flash_utils.clone(),
            message_sender,
            text_input: TextArea::default(),
            preset_command: None,
            is_active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Widget for &mut CommandInput {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_active {
                self.config.theme.focused_border_style
            } else {
                self.config.theme.border_style
            })
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);

        let [preset_command_chunk, text_input_chunk] = Layout::default()
            .horizontal_margin(1)
            .direction(Direction::Horizontal)
            .flex(layout::Flex::Start)
            .spacing(1)
            .constraints(vec![
                Constraint::Length(
                    self.preset_command
                        .clone()
                        .map(|s| s.len() + 2)
                        .unwrap_or(1) as u16,
                ),
                Constraint::Min(1),
            ])
            .areas(inner_area);
        self.text_input.set_style(self.config.theme.command_line);

        block.render(area, buf);
        Text::from(
            self.preset_command
                .clone()
                .unwrap_or(self.config.command_line_prompt_icon.to_string()),
        )
        .style(self.config.theme.header)
        .render(preset_command_chunk, buf);
        self.text_input.render(text_input_chunk, buf);
    }
}

impl crate::messages::MessageReceiver for CommandInput {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        match message {
            Message::Event(Event::Key(key_event))
                if self.is_active && key_event.code == KeyCode::Esc =>
            {
                self.is_active = false;
                self.message_sender.send(Message::SetRawInput(false))?;
            }

            Message::Event(Event::Key(key_event))
                if self.is_active && key_event.code == KeyCode::Enter =>
            {
                let input = self.text_input.lines()[0].to_string();

                match <Command>::from_str(&input) {
                    Ok(command) => {
                        self.is_active = false;
                        self.message_sender.send(Message::SetRawInput(false))?;
                        self.message_sender
                            .send(Message::Command(command.clone()))?;
                    }
                    Err(err) => {
                        self.message_sender.send(Message::Event(Event::Tooltip(
                            Tooltip::from_str(err.to_string().as_str(), TooltipFlavor::Error),
                        )))?;
                        // handle error
                    }
                };
            }

            Message::Event(Event::Key(key_event)) if self.is_active => {
                self.text_input.input(*key_event);
            }

            Message::Command(Command::CommandLineOpen(preset_command)) => {
                self.is_active = true;
                info!("activating raw input");
                self.text_input.select_all();
                self.text_input.delete_char();
                self.message_sender.send(Message::SetRawInput(true))?;
                self.preset_command = preset_command.clone();
            }

            _ => {}
        }

        Ok(())
    }
}
