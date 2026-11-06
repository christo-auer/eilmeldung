use std::fmt::Display;

use news_flash::models::Enclosure;
use strum::EnumMessage;

#[derive(
    Debug,
    Clone,
    Copy,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumMessage,
    strum::AsRefStr,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "lowercase")]
pub enum EnclosureType {
    #[strum(message = "audio", detailed_message = "audio enclosure")]
    Audio,

    #[strum(message = "video", detailed_message = "video enclosure")]
    Video,

    #[strum(message = "image", detailed_message = "image enclosure")]
    Image,
}

impl Display for EnclosureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_message().unwrap().fmt(f) // <- if this fails, it must fail hard
    }
}

impl From<&Enclosure> for EnclosureType {
    fn from(enclosure: &Enclosure) -> Self {
        if enclosure.is_video() {
            EnclosureType::Video
        } else if enclosure.is_audio() {
            EnclosureType::Audio
        } else {
            EnclosureType::Image
        }
    }
}
