use std::sync::Arc;

use crate::prelude::*;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Widget},
};

#[derive(Default)]
pub struct HelpPopup<'a> {
    config: Arc<Config>,
    title: Option<String>,
    contents: Option<Text<'a>>,
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
}

impl<'a> Widget for &HelpPopup<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if let (Some(text), Some(title)) = (self.contents.as_ref(), self.title.as_ref()) {
            let (width, height) = (text.width() + 4, text.height() + 2);

            let [popup_area] = Layout::horizontal([Constraint::Length(width as u16)])
                .flex(Flex::Center)
                .areas::<1>(area);
            let [popup_area, _] =
                Layout::vertical([Constraint::Length(height as u16), Constraint::Length(6)])
                    .flex(Flex::End)
                    .areas::<2>(popup_area);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.config.theme.border_focused())
                .title_top(Line::styled(
                    format!(" {title} "),
                    self.config.theme.header(),
                ))
                .padding(Padding::horizontal(1));

            let inner_area = block.inner(popup_area);

            Widget::render(Clear, popup_area, buf);
            block.render(popup_area, buf);

            text.render(inner_area, buf);
        }
    }
}

impl<'a> MessageReceiver for HelpPopup<'a> {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Event(event) = message {
            use Event::*;
            match event {
                ShowHelpPopup(title, text) => {
                    self.contents = Some(text.to_owned());
                    self.title = Some(title.to_owned());
                }
                HideHelpPopup => self.contents = None,
                _ => {}
            }
        }

        Ok(())
    }
}
