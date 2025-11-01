use ratatui::text::Line;
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum TooltipFlavor {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Tooltip<'a> {
    pub contents: Line<'a>,
    pub flavor: TooltipFlavor,
}

impl<'a> Tooltip<'a> {
    pub fn new(contents: Line<'a>, flavor: TooltipFlavor) -> Self {
        Self { contents, flavor }
    }

    pub fn from_str(content: &str, flavor: TooltipFlavor) -> Self {
        Self {
            contents: Line::from(content.to_string()),
            flavor,
        }
    }

    pub fn to_line(&self, config: &Config) -> Line<'a> {
        let style = match self.flavor {
            TooltipFlavor::Info => config.theme.tooltip_info,
            TooltipFlavor::Warning => config.theme.tooltip_warning,
            TooltipFlavor::Error => config.theme.tooltip_error,
        };
        Line::from(self.contents.clone()).style(style)
    }
}

pub fn tooltip<'a, T: Into<&'a str>>(
    message_sender: &UnboundedSender<Message>,
    content: T,
    flavor: TooltipFlavor,
) -> color_eyre::Result<()> {
    message_sender.send(Message::Event(Event::Tooltip(Tooltip::from_str(
        content.into(),
        flavor,
    ))))?;
    Ok(())
}
