use std::sync::Arc;

use news_flash::models::Feed;
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

#[derive(Debug)]
pub enum PopupEvent {
    // help popup
    ShowHelp(String, Text<'static>),
    ShowModalHelp(String, Text<'static>),
    HideHelp,

    // feed selection
    ShowFeedSelection(Vec<Feed>),
    HideFeedSelection,
}

pub struct PopupManager<'a> {
    message_sender: UnboundedSender<Message>,
    config: Arc<Config>,

    feed_selection_popup: Option<SelectionPopup<'a, Feed, FeedSelectionMapper>>,
    help_popup: Option<HelpPopup<'a>>,
}

impl<'a> PopupManager<'a> {
    pub fn new(message_sender: UnboundedSender<Message>, config: Arc<Config>) -> Self {
        Self {
            message_sender,
            config,
            feed_selection_popup: None,
            help_popup: None,
        }
    }

    async fn handle_popup_event(&mut self, popup_event: &PopupEvent) -> color_eyre::Result<()> {
        use PopupEvent as E;

        match popup_event {
            E::ShowHelp(title, contents) | E::ShowModalHelp(title, contents) => {
                self.help_popup = Some(HelpPopup::new(
                    Arc::clone(&self.config),
                    self.message_sender.clone(),
                    title.to_owned(),
                    contents.to_owned(),
                    matches!(popup_event, E::ShowModalHelp(..)),
                ));
            }
            E::HideHelp => self.help_popup = None,

            E::ShowFeedSelection(feeds) => {
                self.feed_selection_popup = Some(SelectionPopup::new(
                    "Select Feed".to_owned(),
                    feeds.to_vec(),
                    Arc::clone(&self.config),
                    FeedSelectionMapper(Arc::clone(&self.config)),
                    self.message_sender.clone(),
                ))
            }

            E::HideFeedSelection => self.feed_selection_popup = None,
        }

        self.message_sender
            .send(Message::Command(Command::Redraw))?;

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.feed_selection_popup.is_some() || self.help_popup.is_some()
    }
}

impl Widget for &mut PopupManager<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(feed_selection_popup) = self.feed_selection_popup.as_mut() {
            feed_selection_popup.render(area, buf);
        };
        if let Some(help_popup) = self.help_popup.as_mut() {
            help_popup.render(area, buf);
        };
    }
}

impl TermEventHandler for PopupManager<'_> {
    async fn process_term_event(
        &mut self,
        event: &ratatui::crossterm::event::Event,
    ) -> color_eyre::Result<TermEventForwarding> {
        let mut forwarding = TermEventForwarding::PassOn;
        if let Some(feed_selection_popup) = self.feed_selection_popup.as_mut() {
            forwarding = forwarding.pass_to(event, feed_selection_popup).await?;
        };
        if let Some(help_popup) = self.help_popup.as_mut() {
            forwarding = forwarding.pass_to(event, help_popup).await?;
            log::trace!("help popup forwarding: {:?}", forwarding);
        };

        Ok(forwarding)
    }
}

impl MessageReceiver for PopupManager<'_> {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        if let Message::Event(Event::Popup(popup_event)) = message {
            self.handle_popup_event(popup_event).await?;
        };

        if let Some(feed_selection_popup) = self.feed_selection_popup.as_mut() {
            feed_selection_popup.process_message(message).await?;
        }

        if let Some(help_popup) = self.help_popup.as_mut() {
            help_popup.process_message(message).await?;
        }

        Ok(())
    }
}

struct FeedSelectionMapper(Arc<Config>);
impl SelectionPopupMapper for FeedSelectionMapper {
    type Item = Feed;
    fn as_line(&self, value: &Self::Item) -> Line<'static> {
        Line::styled(value.label.clone(), self.0.theme.paragraph())
    }

    fn on_selected_event(&self, value: &Self::Item) -> Event {
        Event::FeedSelected(value.to_owned())
    }
}
