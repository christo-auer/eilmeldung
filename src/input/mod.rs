mod key;

pub mod prelude {
    pub use super::key::{Key, KeySequence};
    pub use super::{InputCommandGenerator, input_reader};
}

use crate::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{info, trace};
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use tokio::sync::mpsc::UnboundedSender;

pub fn input_reader(message_sender: UnboundedSender<Message>) -> color_eyre::Result<()> {
    info!("staring input reader loop");
    loop {
        match ratatui::crossterm::event::read()? {
            ratatui::crossterm::event::Event::Key(key_event) => {
                trace!("crossterm input event: {:?}", key_event);
                message_sender.send(Message::Event(Event::Key(key_event)))?;
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
                self.process_key_event(Some(key_event.clone().into()))
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

    fn generate_input_tooltip(
        &self,
        key_sequence: &KeySequence,
        prefix_matches: &Vec<(&KeySequence, &CommandSequence)>,
    ) {
        if key_sequence.keys.is_empty() {
            return;
        }

        let spans: Vec<Span> = prefix_matches
            .iter()
            .flat_map(|(ks, cs)| {
                let mut keys_reduced = ks.keys.clone();
                keys_reduced.drain(0..key_sequence.keys.len());

                vec![
                    Span::styled(
                        KeySequence { keys: keys_reduced }
                            .to_string()
                            .replace(" ", ""),
                        self.config.theme.tooltip_info.add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("", self.config.theme.tooltip_info),
                    Span::styled(cs.to_string(), self.config.theme.tooltip_info),
                    Span::styled("  ", self.config.theme.tooltip_info),
                ]
            })
            .collect();

        let tooltip = Tooltip::new(Line::from(spans), crate::ui::tooltip::TooltipFlavor::Info);

        let _ = self
            .message_sender
            .send(Message::Event(Event::Tooltip(tooltip)));
    }

    fn process_key_event(&mut self, key: Option<Key>) -> color_eyre::Result<()> {
        let mut aborted = false;
        let now = Instant::now();
        let timeout = now.duration_since(self.last_input_instant) > Duration::from_millis(5000);

        self.last_input_instant = now;

        match key {
            Some(Key::Just(KeyCode::Esc)) => aborted = true,
            Some(key) => {
                self.key_sequence.keys.push(key);
                trace!("current key_sequence: {:?}", self.key_sequence);
            }
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
            let _ = self
                .message_sender
                .send(Message::Event(Event::Tooltip(Tooltip::from_str(
                    " ",
                    TooltipFlavor::Info,
                ))));
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

            self.key_sequence.keys.clear();

            let _ = self
                .message_sender
                .send(Message::Event(Event::Tooltip(tooltip)));
        }

        self.generate_input_tooltip(&self.key_sequence, &prefix_matches);
        Ok(())
    }
}
