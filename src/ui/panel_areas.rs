use crate::prelude::*;
use getset::{Getters, MutGetters};

#[derive(Debug, Default, Clone, Copy, Getters, MutGetters, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PanelAreas {
    pub(super) feed_list: Rect,
    pub(super) articles_list: Rect,
    pub(super) article_content: Rect,
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
        if row >= inner_top && row <= inner_bottom {
            Some(row - inner_top)
        } else {
            None
        }
    }

    /// Returns true if the row is on the horizontal border between the articles list and article content.
    pub(super) fn is_on_article_list_border(&self, col: u16, row: u16) -> bool {
        // The border is at the bottom edge of articles_list / top edge of article_content
        let border_row = self.articles_list.y + self.articles_list.height;
        let in_column_range =
            col >= self.articles_list.x && col < self.articles_list.x + self.articles_list.width;
        row == border_row && in_column_range
    }

    pub(super) fn is_on_feed_list_border(&self, col: u16, row: u16) -> bool {
        // The border is at the bottom edge of articles_list / top edge of article_content
        let border_column = self.feed_list.y + self.feed_list.width;
        let in_row_range =
            row >= self.feed_list.y && row < self.feed_list.y + self.feed_list.height;
        col == border_column && in_row_range
    }

    // pub(super) fn apply_resize(&mut self, panel: Panel, default: Dimension, resize_arg: ResizeArg) {
    //     match panel {
    //         Panel::FeedList => {
    //             let max = self.feed_list.width + self.articles_list.width;
    //             self.feed_list.width = resize_arg
    //                 .apply(Dimension::Length(self.feed_list.width), default, max)
    //                 .into_length(max)
    //                 .raw_value();
    //         }
    //         Panel::ArticleList => {
    //             let max = self.articles_list.height + self.article_content.height;
    //             self.articles_list.height = resize_arg
    //                 .apply(Dimension::Length(self.articles_list.height), default, max)
    //                 .into_length(max)
    //                 .raw_value();
    //         }
    //         Panel::ArticleContent => {
    //             let max = self.articles_list.height + self.article_content.height;
    //             self.article_content.height = resize_arg
    //                 .apply(Dimension::Length(self.article_content.height), default, max)
    //                 .into_length(max)
    //                 .raw_value();
    //         }
    //     }
    // }
}
