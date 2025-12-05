mod key;

pub mod prelude {
    pub use super::key::{Key, KeySequence};
    pub use super::{InputCommandGenerator, input_reader};
}

use crate::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{info, trace};
use ratatui::crossterm::event;
use ratatui::text::{Line, Span, Text};
use tokio::sync::mpsc::UnboundedSender;

pub fn input_reader(message_sender: UnboundedSender<Message>) -> color_eyre::Result<()> {
    info!("starting input reader loop");
    loop {
        match event::read()? {
            event::Event::Key(key_event) => {
                trace!("crossterm input event: {:?}", key_event);
                message_sender.send(Message::Event(Event::Key(key_event)))?;
            }
            event::Event::Resize(width, height) => {
                trace!("resized to {width} {height}");
                message_sender.send(Message::Event(Event::Resized(width, height)))?;
            }
            event => trace!("ignoring event {:?}", event),
        }
    }
}

pub struct InputCommandGenerator {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,
    key_sequence: KeySequence,
    last_input_instant: Instant,
}

impl MessageReceiver for InputCommandGenerator {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        match message {
            Message::Event(Event::Key(key_event)) => {
                self.process_key_event(Some((*key_event).into()))
            }

            Message::Event(Event::Tick) => self.process_key_event(None),

            _ => Ok(()),
        }
    }
}

impl InputCommandGenerator {
    pub fn new(config: Arc<Config>, message_sender: UnboundedSender<Message>) -> Self {
        Self {
            config,
            message_sender,
            key_sequence: KeySequence::default(),
            last_input_instant: Instant::now(),
        }
    }

    fn generate_input_help(
        &self,
        key_sequence: &KeySequence,
        prefix_matches: &Vec<(&KeySequence, &CommandSequence)>,
    ) -> color_eyre::Result<()> {
        if key_sequence.keys.is_empty() {
            return Ok(());
        }

        let lines = Text::from(
            prefix_matches
                .iter()
                .map(|(ks, cs)| {
                    let mut keys_reduced = ks.keys.clone();
                    keys_reduced.drain(0..key_sequence.keys.len());

                    Line::from(vec![
                        Span::styled(
                            KeySequence { keys: keys_reduced }.to_string(),
                            self.config.theme.header(),
                        ),
                        Span::styled("  ", self.config.theme.paragraph()),
                        Span::styled(cs.to_string(), self.config.theme.paragraph()),
                    ])
                })
                .collect::<Vec<Line<'_>>>(),
        );

        self.message_sender
            .send(Message::Event(Event::ShowHelpPopup(
                format!("Input: {}", key_sequence),
                lines,
            )))?;

        Ok(())
    }

    fn process_key_event(&mut self, key: Option<Key>) -> color_eyre::Result<()> {
        let mut aborted = false;
        let now = Instant::now();
        let timeout = now.duration_since(self.last_input_instant) > Duration::from_millis(5000);

        self.last_input_instant = now;

        match key {
            Some(Key::Just(event::KeyCode::Esc)) => aborted = true,
            Some(key) => {
                self.key_sequence.keys.push(key);
                trace!("current key_sequence: {:?}", self.key_sequence);
            }
            None if !timeout => return Ok(()),
            _ => {}
        }

        // get key sequences which have a matching prefix
        let mut prefix_matches = self
            .config
            .input_config
            .input_commands
            .iter()
            .filter(|(other_key_sequence, _)| self.key_sequence.is_prefix_of(other_key_sequence))
            .collect::<Vec<_>>();
        prefix_matches.sort_by(|(ks_1, _), (ks_2, _)| ks_1.keys.len().cmp(&ks_2.keys.len()));

        if let Some(command_sequence) =
            self.config.input_config.input_commands.get(&self.key_sequence) // direct match
                && (prefix_matches.len() == 1 || timeout)
        {
            for command in command_sequence.commands.iter() {
                self.message_sender
                    .send(Message::Command(command.clone()))?;
            }
            self.key_sequence.keys.clear();
            self.message_sender
                .send(Message::Event(Event::HideHelpPopup))?;
        } else if !self.key_sequence.keys.is_empty()
            && (aborted || timeout || prefix_matches.is_empty())
        {
            let tooltip = if aborted {
                Tooltip::from_str("Aborted", crate::ui::tooltip::TooltipFlavor::Info)
            } else {
                Tooltip::from_str(
                    format!("Unknown key sequence: {}", self.key_sequence).as_str(),
                    crate::ui::tooltip::TooltipFlavor::Warning,
                )
            };

            self.message_sender
                .send(Message::Event(Event::HideHelpPopup))?;

            self.key_sequence.keys.clear();

            self.message_sender
                .send(Message::Event(Event::Tooltip(tooltip)))?;
        }

        self.generate_input_help(&self.key_sequence, &prefix_matches)?;
        Ok(())
    }
}
