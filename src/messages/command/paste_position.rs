use std::fmt::Display;
use strum::EnumMessage;

#[derive(Debug, Clone, Copy, Default, strum::EnumString, strum::EnumIter, strum::EnumMessage)]
#[strum(serialize_all = "lowercase")]
pub enum PastePosition {
    #[default]
    #[strum(
        message = "after",
        detailed_message = "position after the current element"
    )]
    After,
    #[strum(
        message = "before",
        detailed_message = "position before the current element"
    )]
    Before,
}

impl Display for PastePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}
