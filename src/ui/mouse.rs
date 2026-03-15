use crate::prelude::*;

use getset::{Getters, MutGetters};
use ratatui::prelude::Rect;

/// Stores the last rendered areas of the three main panels for mouse hit-testing.
#[derive(Default, Clone, Copy, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PanelAreas {
    feed_list: Rect,
    articles_list: Rect,
    article_content: Rect,
}

impl PanelAreas {
    pub(super) fn panel_at(&self, col: u16, row: u16) -> Option<Panel> {
        if self.feed_list.contains((col, row).into()) {
            Some(Panel::FeedList)
        } else if self.articles_list.contains((col, row).into()) {
            Some(Panel::ArticleList)
        } else if self.article_content.contains((col, row).into()) {
            Some(Panel::ArticleContent)
        } else {
            None
        }
    }

    /// Returns the row offset relative to the inner area of the articles panel (excluding border).
    pub(super) fn article_row_offset(&self, row: u16) -> Option<u16> {
        let area = self.articles_list;
        // Account for the border (1 row top)
        let inner_top = area.y + 1;
        let inner_bottom = area.y + area.height.saturating_sub(1);
        if row >= inner_top && row < inner_bottom {
            Some(row - inner_top)
        } else {
            None
        }
    }

    /// Returns true if the row is on the horizontal border between the articles list and article content.
    pub(super) fn is_on_horizontal_border(&self, col: u16, row: u16) -> bool {
        // The border is at the bottom edge of articles_list / top edge of article_content
        let border_row = self.articles_list.y + self.articles_list.height;
        let in_column_range =
            col >= self.articles_list.x && col < self.articles_list.x + self.articles_list.width;
        row == border_row && in_column_range
    }
}
