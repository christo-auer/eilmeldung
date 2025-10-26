mod key;

pub mod prelude {
    pub use super::InputCommandHandler;
    pub use super::key::{Key, KeySequence};
}

use crate::prelude::*;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, trace, warn};
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

// this is heavily inspired by spotify-player's key.rs

pub struct InputCommandHandler {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,
    raw_input: Arc<Mutex<bool>>,
}

impl InputCommandHandler {
    fn new(
        config: Arc<Config>,
        message_sender: UnboundedSender<Message>,
        raw_input: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            config,
            message_sender,
            raw_input: raw_input.clone(),
        }
    }

    pub fn spawn_input_command_handler(
        config: Arc<Config>,
        message_sender: UnboundedSender<Message>,
        raw_input: Arc<Mutex<bool>>,
    ) {
        tokio::spawn(async move {
            let mut handler = Self::new(config, message_sender, raw_input);
            if let Err(error) = handler.handle_input_commands().await {
                warn!("error handling input {error}");
            }
        });
    }

    pub async fn handle_input_commands(&mut self) -> color_eyre::Result<()> {
        debug!("Input processing loop started");
        let mut key_sequence = KeySequence::default();
        loop {
            let mut timeout = false;
            let mut aborted = false;

            if let Ok(event_available) =
                ratatui::crossterm::event::poll(Duration::from_millis(5000))
            {
                if event_available {
                    match ratatui::crossterm::event::read()? {
                        ratatui::crossterm::event::Event::Key(key_event) => {
                            {
                                let raw_input = self.raw_input.lock().await;
                                if *raw_input {
                                    trace!("sending raw key event: {:?}", key_event);
                                    self.message_sender
                                        .send(Message::Event(Event::Key(key_event)))?;
                                    continue;
                                }
                            }

                            let key: Key = key_event.into();

                            if key == Key::Just(KeyCode::Esc) {
                                aborted = true;
                            } else {
                                key_sequence.keys.push(key);
                                trace!("current key_sequence: {:?}", key_sequence);
                            }
                        }
                        _ => {
                            trace!("Non-key event ignored");
                        }
                    }
                } else {
                    trace!("input timeout");
                    timeout = true;
                }
            }

            {
                let raw_input = self.raw_input.lock().await;
                if *raw_input {
                    continue;
                }
            }

            // get key sequences which have a matching prefix
            let mut prefix_matches = self
                .config
                .input_config
                .input_commands
                .iter()
                .filter(|(other_key_sequence, _)| key_sequence.is_prefix_of(other_key_sequence))
                .collect::<Vec<_>>();
            prefix_matches.sort_by(|(ks_1, _), (ks_2, _)| ks_1.keys.len().cmp(&ks_2.keys.len()));

            trace!("prefix matches of input: {}", prefix_matches.len());

            if let Some(command_sequence) =
                self.config.input_config.input_commands.get(&key_sequence) // direct match
                    && (prefix_matches.len() == 1 || timeout)
            {
                for command in command_sequence.commands.iter() {
                    self.message_sender
                        .send(Message::Command(command.clone()))?;
                }
                key_sequence.keys.clear();
                let _ =
                    self.message_sender
                        .send(Message::Event(Event::Tooltip(Tooltip::from_str(
                            " ",
                            TooltipFlavor::Info,
                        ))));
            } else if !key_sequence.keys.is_empty()
                && (aborted || timeout || prefix_matches.is_empty())
            {
                let tooltip = if aborted {
                    Tooltip::from_str("Aborted", crate::ui::tooltip::TooltipFlavor::Info)
                } else {
                    Tooltip::from_str(
                        format!("Unknown key sequence: {key_sequence}").as_str(),
                        crate::ui::tooltip::TooltipFlavor::Warning,
                    )
                };

                key_sequence.keys.clear();

                let _ = self
                    .message_sender
                    .send(Message::Event(Event::Tooltip(tooltip)));
            }

            self.generate_input_tooltip(&key_sequence, &prefix_matches);
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
}
