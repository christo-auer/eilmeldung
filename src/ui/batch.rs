use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

pub struct BatchProcessor {
    command_queue: (UnboundedSender<Command>, UnboundedReceiver<Command>),
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            command_queue: mpsc::unbounded_channel(),
        }
    }

    pub async fn next(&mut self) -> Option<Command> {
        self.command_queue.1.recv().await
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
            // enqueue commands
            commands
                .iter()
                .try_for_each(|command| self.command_queue.0.send(command.to_owned()))?;
        }

        Ok(())
    }
}
