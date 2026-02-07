use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

pub struct BatchProcessor {
    // config: Arc<Config>,
    // news_flash_utils: Arc<NewsFlashUtils>,
    // message_sender: UnboundedSender<Message>,
    command_queue: (UnboundedSender<Command>, UnboundedReceiver<Command>),
}

impl BatchProcessor {
    pub fn new(// config: Arc<Config>,
        // news_flash_utils: Arc<NewsFlashUtils>,
        // message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            // config,
            // news_flash_utils,
            // message_sender,
            command_queue: mpsc::unbounded_channel(),
        }
    }

    pub async fn next(&mut self) -> Option<Command> {
        self.command_queue.1.recv().await
    }

    pub fn has_commands(&self) -> bool {
        !self.command_queue.1.is_empty()
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
