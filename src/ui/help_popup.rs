use std::{borrow::Cow, sync::Arc};

use crate::prelude::*;
use ratatui::{
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget},
};

#[derive(Default)]
pub struct HelpPopup<'a> {
    config: Arc<Config>,
    title: Option<String>,
    contents: Option<Vec<Line<'a>>>,
}

impl<'a> HelpPopup<'a> {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            title: None,
            contents: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.contents.is_some()
    }

    pub fn needed_height(&self) -> u16 {
        self.contents
            .as_ref()
            .map(|lines| lines.len() as u16)
            .unwrap_or(0)
            + 2
    }
}

impl<'a> Widget for &HelpPopup<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if let (Some(contents), Some(title)) = (self.contents.as_ref(), self.title.as_ref()) {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.config.theme.focused_border_style)
                .title_top(Line::styled(format!(" {title} "), self.config.theme.header))
                .padding(Padding::horizontal(1));

            let inner_area = block.inner(area);

            Widget::render(Clear, area, buf);
            block.render(area, buf);

            let paragraph = Paragraph::new(contents.to_owned());
            paragraph.render(inner_area, buf);
        }
    }
}

impl<'a> MessageReceiver for HelpPopup<'a> {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Event(event) = message {
            use Event::*;
            match event {
                ShowHelpPopup(title, lines) => {
                    self.contents = Some(lines.to_owned());
                    self.title = Some(title.to_owned());
                }
                HideHelpPopup => self.contents = None,
                _ => {}
            }
        }

        Ok(())
    }
}
