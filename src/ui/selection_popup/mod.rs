use std::sync::Arc;

use crate::prelude::*;
use ratatui::layout::Flex;
use tokio::sync::mpsc::UnboundedSender;

use ratatui::crossterm::event::Event as TermEvent;

pub struct SelectionPopup<'a, T, M: SelectionPopupMapper<Item = T>> {
    message_sender: UnboundedSender<Message>,

    config: Arc<Config>,
    contents: Vec<T>,
    mapper: M,

    title: String,

    list_state: ListState,
    list: List<'a>,

    longest_width: u16,
}

pub trait SelectionPopupMapper {
    type Item;
    fn as_line(&self, value: &Self::Item) -> Line<'static>;
    fn on_selected_event(&self, value: &Self::Item) -> Event;
}

impl<'a, T, M: SelectionPopupMapper<Item = T>> SelectionPopup<'a, T, M> {
    pub fn new(
        title: String,
        contents: Vec<T>,
        config: Arc<Config>,
        mapper: M,
        message_sender: UnboundedSender<Message>,
    ) -> SelectionPopup<'a, T, M> {
        let list_items: Vec<Line<'_>> = contents.iter().map(|item| mapper.as_line(item)).collect();

        let longest_width = list_items
            .iter()
            .map(|item| item.width() as u16)
            .max()
            .unwrap_or_default();

        let selected_style = config.theme.selected(&Default::default());
        let mut list_state = ListState::default();
        list_state.select_first();

        Self {
            title,
            contents,
            mapper,
            config,
            longest_width,
            list: List::new(list_items).highlight_style(selected_style),
            message_sender,
            list_state,
        }
    }

    fn on_submit(&mut self) -> color_eyre::Result<()> {
        if let Some(selected_index) = self.list_state.selected()
            && let Some(item) = self.contents.get(selected_index)
        {
            let event = self.mapper.on_selected_event(item);
            self.message_sender.send(Message::Event(event))?;
            self.message_sender
                .send(Message::Event(Event::Popup(PopupEvent::HideFeedSelection)))?;
        } else {
            return Err(color_eyre::eyre::eyre!("invalid selection"));
        }

        Ok(())
    }
}

impl<'a, T, M: SelectionPopupMapper<Item = T>> Widget for &mut SelectionPopup<'a, T, M> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (width, height) = (
            self.longest_width.saturating_add(4).min(area.width),
            (self.contents.len() as u16)
                .saturating_add(2)
                .min(area.height),
        );

        let [popup_area] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas::<1>(area);
        let [_, popup_area, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(height),
            Constraint::Length(5),
        ])
        .flex(Flex::End)
        .areas::<3>(popup_area);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .style(Style::default().bg(*self.config.theme.color_palette().background()))
            .border_type(self.config.border_theme.focused)
            .border_style(self.config.theme.border_focused())
            .title_top(Line::styled(
                format!(" {} ", self.title),
                self.config.theme.header(),
            ))
            .padding(Padding::horizontal(1));

        if self.config.shadows {
            block = block.shadow(Shadow::light_shade());
        }

        let contents_area = block.inner(popup_area);

        Widget::render(Clear, popup_area, buf);
        block.render(popup_area, buf);

        StatefulWidget::render(&self.list, contents_area, buf, &mut self.list_state);
    }
}

impl<'a, T, M: SelectionPopupMapper<Item = T>> MessageReceiver for SelectionPopup<'a, T, M> {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Event(Event::ConfigReloaded(config)) = message {
            self.config = Arc::clone(config);
        };

        Ok(())
    }
}

impl<'a, T, M: SelectionPopupMapper<Item = T>> TermEventHandler for SelectionPopup<'a, T, M> {
    async fn process_term_event(
        &mut self,
        event: &TermEvent,
    ) -> color_eyre::Result<TermEventForwarding> {
        let TermEvent::Key(key_event) = event else {
            return Ok(TermEventForwarding::Consumed);
        };

        let mut redraw = true;

        match self
            .config
            .input_config
            .match_single_key_to_single_command(&Key::from(*key_event))
        {
            Some(Command::NavigateUp) => self.list_state.select_previous(),
            Some(Command::NavigateDown) => self.list_state.select_next(),
            Some(Command::NavigatePageDown) => self
                .list_state
                .scroll_down_by(self.config.input_config.scroll_amount as u16),
            Some(Command::NavigatePageUp) => self
                .list_state
                .scroll_up_by(self.config.input_config.scroll_amount as u16),
            Some(Command::InputAbort) => self
                .message_sender
                .send(Message::Event(Event::Popup(PopupEvent::HideFeedSelection)))?,
            Some(Command::InputSubmit) => self.on_submit()?,
            _ => redraw = false,
        }

        if redraw {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(TermEventForwarding::Consumed)
    }
}
