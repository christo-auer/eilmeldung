use std::sync::Arc;

use log::info;
use ratatui::{
    crossterm::event::KeyCode,
    widgets::{Block, BorderType, Borders, Widget},
};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use tui_textarea::TextArea;

use crate::{
    app::AppState,
    commands::{Command, Event, Message, MessageReceiver},
    config::Config,
    newsflash_utils::NewsFlashUtils,
};

pub struct CommandInput {
    config: Arc<Config>,

    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    text_input: TextArea<'static>,

    is_focused: bool,

    previous_app_state: Option<AppState>,
}

impl CommandInput {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            text_input: TextArea::default(),
            is_focused: false,
            previous_app_state: None,
        }
    }
}

impl Widget for &mut CommandInput {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.text_input.set_block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(if self.is_focused {
                    self.config.theme.focused_border_style
                } else {
                    self.config.theme.border_style
                })
                .border_type(BorderType::Rounded)
                .title_bottom("Command"),
        );

        self.text_input.render(area, buf);
    }
}

impl MessageReceiver for CommandInput {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        info!("input_command processing {:?}", message);
        match message {
            Message::Event(Event::Key(key_event))
                if self.is_focused && key_event.code == KeyCode::Esc =>
            {
                self.message_sender.send(Message::SetRawInput(false))?;
                self.message_sender
                    .send(Message::Command(Command::PanelFocus(
                        self.previous_app_state.unwrap_or(AppState::FeedSelection),
                    )))?;
            }

            Message::Event(Event::Key(key_event)) if self.is_focused => {
                self.text_input.input(*key_event);
            }

            Message::Event(Event::ApplicationStateChanged(new_state)) => match new_state {
                AppState::CommandInput => {
                    self.is_focused = true;
                    info!("activating raw input");
                    self.message_sender.send(Message::SetRawInput(true))?;
                }

                _ => {
                    self.is_focused = false;
                    self.previous_app_state = Some(*new_state);
                }
            },

            _ => {}
        }

        Ok(())
    }
}
