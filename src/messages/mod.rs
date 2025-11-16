pub mod command;
pub mod event;

pub mod prelude {
    pub use super::command::{ActionScope, ActionSetReadTarget, Command, CommandSequence};
    pub use super::event::{AsyncOperationError, Event};
    pub use super::{Message, MessageReceiver};
}

use crate::prelude::*;

#[derive(Debug)]
pub enum Message {
    Command(Command),
    Event(Event),
}

pub trait MessageReceiver {
    fn process_command(
        &mut self,
        message: &Message,
    ) -> impl std::future::Future<Output = color_eyre::Result<()>> + Send;
}
