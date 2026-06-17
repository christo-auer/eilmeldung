use paste::paste;

use serde::Deserialize;
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IconSet {
    preset: IconSetPreset,
    // Core UI Icons
    offline: Option<char>,
    all: Option<char>,
    tag: Option<char>,
    info: Option<char>,
    warning: Option<char>,
    error: Option<char>,
    read: Option<char>,
    unread: Option<char>,
    marked: Option<char>,
    unmarked: Option<char>,
    enclosure_video: Option<char>,
    enclosure_audio: Option<char>,
    enclosure_image: Option<char>,
    flagged: Option<char>,
    command_line_prompt: Option<char>,
    image: Option<char>,
    url: Option<char>,
    big_icon_left: Option<char>,
    big_icon_right: Option<char>,

    // Feed and List Labels
    feeds: Option<char>,
    last_synced: Option<char>,
    feed: Option<char>,
    category: Option<char>,
    categories: Option<char>,
    tags: Option<char>,
    query: Option<char>,
    // Sorting and Filtering
    sort_ascending: Option<char>,
    sort_descending: Option<char>,
    filter_reverse: Option<char>,
    sort_normal: Option<char>,
    piped: Option<char>,
    // Batch Operations
    batch: Option<char>,
    check_progress: Option<char>,
    check_complete: Option<char>,
    // Status Bar
    status_bar_left: Option<char>,
    status_bar_right: Option<char>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IconSetPreset {
    #[default]
    Nerd,
    Ascii,
}

macro_rules! icon_property {
    ($name: ident, $nerd:literal, $ascii:literal) => {
        paste! {
            pub fn [<$name _icon>](&self) -> char {
                self.$name.unwrap_or(match self.preset {
                    IconSetPreset::Nerd => $nerd,
                    IconSetPreset::Ascii => $ascii,
                })
            }
        }
    };
}

impl IconSet {
    // Core UI Icons
    icon_property!(offline, '', '?');
    icon_property!(read, '', ' ');
    icon_property!(unread, '', '•');
    icon_property!(all, '', '≡');
    icon_property!(marked, '', '♥');
    icon_property!(unmarked, ' ', ' ');
    icon_property!(tag, '󰓹', '¶');
    icon_property!(command_line_prompt, '', '>');
    icon_property!(info, '', 'I');
    icon_property!(warning, '', 'W');
    icon_property!(error, '', 'E');
    icon_property!(enclosure_video, '', 'V');
    icon_property!(enclosure_audio, '', 'A');
    icon_property!(enclosure_image, '', 'I');
    icon_property!(flagged, '', '*');
    icon_property!(image, '', 'I');
    icon_property!(url, '', 'L');
    icon_property!(big_icon_left, '', ' ');
    icon_property!(big_icon_right, '', ' ');
    // Feed and List Labels
    icon_property!(feeds, '󱀂', 'A');
    icon_property!(last_synced, '', 'S');
    icon_property!(feed, '', 'F');
    icon_property!(category, '󰉋', 'C');
    icon_property!(categories, '󰉓', 'C');
    icon_property!(tags, '󰓻', 'T');
    icon_property!(query, '', 'Q');
    icon_property!(piped, '󰟥', '|');
    // Sorting and Filtering
    icon_property!(sort_ascending, '󰒼', '^');
    icon_property!(sort_descending, '󰒽', 'v');
    icon_property!(filter_reverse, '󰒿', '<');
    icon_property!(sort_normal, '󰌼', '>');
    // Batch Operations
    icon_property!(batch, '', '=');
    icon_property!(check_progress, '', '+');
    icon_property!(check_complete, '', '*');
    // Status Bar
    icon_property!(status_bar_left, '', ' ');
    icon_property!(status_bar_right, '', ' ');
}
