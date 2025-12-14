pub mod command;
pub mod event;

pub mod prelude {
    pub use super::command::prelude::*;
    pub use super::event::{AsyncOperationError, Event};
    pub use super::{Message, MessageReceiver};
}

use crate::prelude::*;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // OK in this case as the number of allocation is
                                     // comparatively small and the enums are short-lived
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
