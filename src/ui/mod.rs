pub mod feeds_list;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::app::App;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = format!(
        //     " {} - {} ",
        //     env!("CARGO_PKG_NAME"),
        //     env!("CARGO_PKG_VERSION")
        // );
        // let block = Block::bordered()
        //     .title(title)
        //     .title_alignment(ratatui::layout::Alignment::Left)
        //     .border_type(ratatui::widgets::BorderType::Rounded);

        // block.render(area, buf);

        let feeds_width = self.config.theme.feeds_width;
        let articles_width = 100 - feeds_width;
        let feed_articles_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages(vec![
                feeds_width,
                articles_width,
            ]))
            .split(area);

        self.feed_list
            .render(*feed_articles_chunks.first().unwrap(), buf);
    }
}
