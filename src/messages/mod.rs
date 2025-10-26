pub mod command;
pub mod event;

pub mod prelude {
    pub use super::command::{Command, CommandSequence};
    pub use super::event::Event;
    pub use super::{Message, MessageReceiver};
}

use crate::prelude::*;

#[derive(Debug)]
pub enum Message {
    Command(Command),
    Event(Event),
    SetRawInput(bool),
}

pub trait MessageReceiver {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()>;
}
