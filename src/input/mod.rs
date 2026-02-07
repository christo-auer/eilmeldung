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
use throbber_widgets_tui::{Throbber, ThrobberState, VERTICAL_BLOCK};
use tokio::sync::mpsc::UnboundedSender;

pub fn input_reader(message_sender: UnboundedSender<Message>) -> color_eyre::Result<()> {
    info!("starting input reader loop");
    loop {
        if !event::poll(Duration::from_millis(100))? {
            if message_sender.is_closed() {
                return Ok(());
            }
            continue;
        }

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
            Message::Command(Command::HelpInput) => self.show_help_input(),
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
        key_sequence: Option<&KeySequence>,
        prefix_matches: &[(&KeySequence, &CommandSequence)],
        timeout_ratio: f32,
    ) -> color_eyre::Result<()> {
        if let Some(key_sequence) = key_sequence
            && key_sequence.keys.is_empty()
        {
            return Ok(());
        }

        let key_seq_len = key_sequence.map(|ks| ks.keys.len()).unwrap_or(0);

        let max_key_length = prefix_matches
            .iter()
            .map(|(ks, _)| ks.to_string().len().saturating_sub(key_seq_len))
            .max()
            .unwrap_or_default();

        let lines = Text::from(
            prefix_matches
                .iter()
                .map(|(ks, cs)| {
                    let mut keys_reduced = ks.keys.clone();

                    keys_reduced.drain(0..key_seq_len);

                    let key_entry = if keys_reduced.is_empty() {
                        let throbber_set = VERTICAL_BLOCK;
                        let len = throbber_set.symbols.len();
                        let throbber = Throbber::default()
                            .throbber_style(self.config.theme.header())
                            .throbber_set(throbber_set)
                            .use_type(throbber_widgets_tui::WhichUse::Spin);
                        let mut throbber_state = ThrobberState::default();
                        throbber_state.calc_step(len as i8);
                        throbber_state
                            .calc_step(-((len as f32 * timeout_ratio + 1.0).floor()) as i8);
                        throbber.to_symbol_span(&throbber_state)
                    } else {
                        let keys = KeySequence { keys: keys_reduced }.to_string();
                        Span::styled(keys, self.config.theme.header())
                    };

                    let filler = Span::styled(
                        " ".repeat(max_key_length.saturating_sub(key_entry.width()) + 1),
                        self.config.theme.header(),
                    );

                    Line::from(vec![
                        key_entry,
                        filler,
                        Span::styled(cs.to_string(), self.config.theme.paragraph()),
                    ])
                })
                .collect::<Vec<Line<'_>>>(),
        );

        match key_sequence {
            Some(key_sequence) => {
                self.message_sender
                    .send(Message::Event(Event::ShowHelpPopup(
                        format!("Input: {}", key_sequence),
                        lines,
                    )))?
            }
            None => self
                .message_sender
                .send(Message::Event(Event::ShowModalHelpPopup(
                    "Input Mappings".to_owned(),
                    lines,
                )))?,
        }

        Ok(())
    }

    fn process_key_event(&mut self, key: Option<Key>) -> color_eyre::Result<()> {
        let mut aborted = false;
        let mut submit = false;
        let now = Instant::now();

        let command = key.as_ref().and_then(|key| {
            self.config
                .input_config
                .match_single_key_to_single_command(key)
        });

        match command {
            Some(Command::InputAbort) => {
                aborted = true;
                self.last_input_instant = now;
            }
            Some(Command::InputSubmit) => {
                submit = true;
                self.last_input_instant = now;
            }
            _ => {
                if let Some(key) = key {
                    self.last_input_instant = now;
                    self.key_sequence.keys.push(key);
                    trace!("current key_sequence: {:?}", self.key_sequence);
                }
            }
        }

        let duration = now.duration_since(self.last_input_instant);
        let timeout = duration > Duration::from_millis(self.config.input_config.timeout_millis);
        let timeout_ratio =
            duration.as_millis() as f32 / self.config.input_config.timeout_millis as f32;

        // get key sequences which have a matching prefix
        let mut prefix_matches = self
            .config
            .input_config
            .mappings
            .iter()
            .filter(|(other_key_sequence, _)| self.key_sequence.is_prefix_of(other_key_sequence))
            .collect::<Vec<_>>();
        prefix_matches.sort_by(|(ks_1, _), (ks_2, _)| ks_1.keys.len().cmp(&ks_2.keys.len()));

        if key.is_none() && !timeout && !self.key_sequence.keys.is_empty() {
            self.generate_input_help(Some(&self.key_sequence), &prefix_matches, timeout_ratio)?;
            return Ok(());
        }

        if let Some(command_sequence) =
            self.config.input_config.mappings.get(&self.key_sequence) // direct match
                && (prefix_matches.len() == 1 || timeout || submit)
        {
            if command_sequence.commands.len() > 1 {
                // create a batch of commands
                self.message_sender.send(Message::Batch(
                    command_sequence.commands.iter().cloned().collect(),
                ))?;
            } else if command_sequence.commands.len() == 1 {
                // send single command
                self.message_sender.send(Message::Command(
                    command_sequence
                        .commands
                        .first()
                        .unwrap_or(&Command::NoOperation)
                        .to_owned(),
                ))?;
            }
            self.key_sequence.keys.clear();
            self.message_sender
                .send(Message::Event(Event::HideHelpPopup))?;
        } else if !self.key_sequence.keys.is_empty()
            && (aborted || timeout || prefix_matches.is_empty())
        {
            let tooltip = if aborted {
                Tooltip::from_str("Aborted", TooltipFlavor::Info)
            } else {
                Tooltip::from_str(
                    format!("Unknown key sequence: {}", self.key_sequence).as_str(),
                    TooltipFlavor::Warning,
                )
            };

            self.message_sender
                .send(Message::Event(Event::HideHelpPopup))?;

            self.key_sequence.keys.clear();

            self.message_sender
                .send(Message::Event(Event::Tooltip(tooltip)))?;
        }

        self.generate_input_help(Some(&self.key_sequence), &prefix_matches, timeout_ratio)?;

        Ok(())
    }

    fn show_help_input(&self) -> Result<(), color_eyre::eyre::Error> {
        self.generate_input_help(
            None,
            &self
                .config
                .input_config
                .mappings
                .iter()
                .collect::<Vec<(&KeySequence, &CommandSequence)>>(),
            0f32,
        )
    }
}
