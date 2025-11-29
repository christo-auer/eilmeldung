use crate::{messages::command::CommandParseError, prelude::*};

use std::{str::FromStr, sync::Arc};

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

pub struct CommandInput {
    config: Arc<Config>,

    _news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    text_input: TextArea<'static>,
    command_hint: Option<Line<'static>>,

    history: Vec<String>,
    history_index: usize,

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
            history: Vec::default(),
            command_hint: None,
            history_index: 0,
            is_active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    fn to_command(&self) -> Result<Command, CommandParseError> {
        let input = self.text_input.lines()[0].as_str();

        Ok(Command::parse(input)?)
    }

    fn on_submit(&mut self) -> color_eyre::Result<()> {
        match self.to_command() {
            Ok(command) => {
                self.is_active = false;
                self.message_sender
                    .send(Message::Command(command.clone()))?;
            }
            Err(err) => {
                self.message_sender
                    .send(Message::Event(Event::Tooltip(Tooltip::from_str(
                        err.to_string().as_str(),
                        TooltipFlavor::Error,
                    ))))?;
                // handle error
            }
        };
        Ok(())
    }

    fn clear(&mut self, s: &str) {
        self.text_input.select_all();
        self.text_input.delete_char();
        self.text_input.insert_str(s);
    }

    fn on_history(&mut self, index: usize) {
        self.history_index = index;
        let history_entry = self.history.get(index).unwrap().to_string();
        self.clear(&history_entry);
    }

    fn on_history_previous(&mut self) {
        if let Some(index) = self.history[0..self.history_index]
            .iter()
            .rposition(|entry| {
                entry.starts_with(self.history.last().map(String::as_str).unwrap_or_default())
            })
        {
            self.on_history(index);
        }
    }

    fn on_history_next(&mut self) {
        if let Some(index) = self.history[self.history_index + 1..]
            .iter()
            .position(|entry| {
                entry.starts_with(self.history.last().map(String::as_str).unwrap_or_default())
            })
        {
            self.on_history(index + self.history_index + 1);
        }
    }
    //
    fn on_parse_error(&mut self, error: &CommandParseError) {
        use CommandParseError as E;
        let message = match error {
            E::WordExpected(expected) => expected.to_owned(),
            E::SomethingExpected(expected) => expected.to_owned(),
            E::NothingExcepted(not_expected) => format!("expected nothing but got {not_expected}"),
            E::ArticleQueryExpected(query_parse_error) => query_parse_error.to_string(),

            error => error.to_string(),
        };

        self.command_hint = Some(Line::styled(
            format!(" {message} "),
            self.config.theme.highlight_color,
        ));
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
            .title_bottom(self.command_hint.clone().unwrap_or_default())
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);

        let [preset_command_chunk, text_input_chunk] = Layout::default()
            .horizontal_margin(1)
            .direction(Direction::Horizontal)
            .flex(layout::Flex::Start)
            .spacing(1)
            .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
            .areas(inner_area);
        self.text_input.set_style(self.config.theme.command_input);

        block.render(area, buf);
        Text::from(self.config.command_line_prompt_icon.to_string())
            .style(self.config.theme.header)
            .render(preset_command_chunk, buf);
        self.text_input.render(text_input_chunk, buf);
    }
}

impl crate::messages::MessageReceiver for CommandInput {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let config = &self.config.input_config.command_line;
        match message {
            Message::Event(Event::Key(key_event)) if self.is_active => {
                let key: Key = (*key_event).into();

                if config.abort.contains(&key) {
                    self.history.remove(self.history.len() - 1);
                    self.is_active = false;
                } else if config.submit.contains(&key) {
                    self.on_submit()?;
                } else if config.clear.contains(&key) {
                    self.clear("");
                } else if config.history_next.contains(&key) {
                    self.on_history_next();
                } else if config.history_previous.contains(&key) {
                    self.on_history_previous();
                } else if self.text_input.input(*key_event) {
                    self.history_index = self.history.len() - 1;
                    *self.history.last_mut().unwrap() = self.text_input.lines()[0].to_string();

                    match self.to_command() {
                        Ok(command) => {
                            self.command_hint = Some(Line::styled(
                                command.to_string(),
                                self.config.theme.normal_color,
                            ))
                        }
                        Err(error) => self.on_parse_error(&error),
                    }
                }
            }

            Message::Command(Command::CommandLineOpen(preset_command)) => {
                log::trace!("history: {:?}", self.history);

                self.is_active = true;
                self.clear("");

                let preset_command = preset_command
                    .as_deref()
                    .map(|preset_command| format!("{} ", preset_command))
                    .unwrap_or_default();

                self.history.push(preset_command.to_string());
                self.history_index = self.history.len() - 1;
                self.text_input.insert_str(preset_command);
            }

            _ => {}
        }

        Ok(())
    }
}
