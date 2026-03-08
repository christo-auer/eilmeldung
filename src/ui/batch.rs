use std::sync::Arc;

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

pub struct BatchProcessor {
    config: Arc<Config>,
    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,
    command_queue: (UnboundedSender<Command>, UnboundedReceiver<Command>),
    current_command_async: bool,
    show_popup: bool,
    popup_strings: Vec<String>,
    current_command_index: usize,
}

impl BatchProcessor {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config,
            message_sender,
            news_flash_utils,
            command_queue: mpsc::unbounded_channel(),
            current_command_async: false,
            show_popup: false,
            popup_strings: Default::default(),
            current_command_index: 0,
        }
    }

    pub async fn next(&mut self) -> color_eyre::Result<Command> {
        let command = self
            .command_queue
            .1
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("batch command receiver closed"))?;

        self.current_command_async = command.is_async();

        if self.show_popup {
            self.current_command_index += 1;
            self.update_popup()?;
        }

        if !self.has_commands() && self.show_popup {
            self.show_popup = false;
            self.hide_popup()?;
        }

        Ok(command)
    }

    pub fn waiting_for_async_operation(&self) -> bool {
        self.current_command_async && self.news_flash_utils.is_async_operation_running()
    }

    pub fn show_popup(&mut self) {
        self.show_popup = true;
    }

    fn update_popup(&self) -> color_eyre::Result<()> {
        let mut lines = self
            .popup_strings
            .iter()
            .enumerate()
            .map(|(index, command)| {
                let (state, style) = if index < self.current_command_index.saturating_sub(1) {
                    ('', self.config.theme.inactive())
                } else if index == self.current_command_index.saturating_sub(1) {
                    (
                        '',
                        self.config
                            .theme
                            .highlighted(&self.config.theme.paragraph()),
                    )
                } else {
                    (' ', self.config.theme.paragraph())
                };
                Line::from(vec![
                    Span::styled(
                        format!(" {state} ").to_owned(),
                        self.config.theme.paragraph(),
                    ),
                    Span::styled(command.to_owned(), style),
                ])
            })
            .collect::<Vec<Line>>();
        lines.pop();

        self.message_sender
            .send(Message::Event(Event::ShowHelpPopup(
                "".to_owned(),
                Text::from(lines),
            )))?;

        Ok(())
    }

    fn hide_popup(&self) -> color_eyre::Result<()> {
        self.message_sender
            .send(Message::Event(Event::HideHelpPopup))?;

        Ok(())
    }

    pub fn has_commands(&self) -> bool {
        !self.command_queue.1.is_empty()
    }

    pub fn abort(&mut self) {
        while self.has_commands() {
            let _ = self.command_queue.1.try_recv();
        }
    }
}

impl MessageReceiver for BatchProcessor {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Batch(commands) = message {
            // no commands in batch? -> return
            if commands.is_empty() {
                self.show_popup = false;
                return Ok(());
            }

            // if its just one command, execute it directly
            if commands.len() == 1
                && let Some(command) = commands.first()
            {
                self.show_popup = false;
                self.message_sender
                    .send(Message::Command(command.to_owned()))?;
                return Ok(());
            }

            // add sentinel value for "end of batch"
            let mut commands = commands.to_vec();
            commands.push(Command::NoOperation);

            self.popup_strings = commands.iter().map(|command| command.to_string()).collect();
            self.current_command_index = 0;

            // enqueue commands
            commands
                .into_iter()
                .try_for_each(|command| self.command_queue.0.send(command))?;
        }

        Ok(())
    }
}
