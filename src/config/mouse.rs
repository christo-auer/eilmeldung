use indexmap::IndexMap;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

use crate::{config::input_mappings, prelude::*};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct MouseConfig {
    pub enable: bool,
    pub content_resize: bool,
    pub toggle_tree_on_click: bool,
    feeds_mapping: IndexMap<MouseInput, CommandSequence>,
    articles_mapping: IndexMap<MouseInput, CommandSequence>,
    content_mapping: IndexMap<MouseInput, CommandSequence>,
}

impl MouseConfig {
    pub fn get_mapping(&self, mouse_input: MouseInput, panel: Panel) -> Option<&CommandSequence> {
        match panel {
            Panel::FeedList => &self.feeds_mapping,
            Panel::ArticleList => &self.articles_mapping,
            Panel::ArticleContent => &self.content_mapping,
        }
        .get(&mouse_input)
    }

    pub async fn validate(&mut self) -> color_eyre::Result<()> {
        let default = Self::default();
        Self::prepare_mapping(default.feeds_mapping, &mut self.feeds_mapping);
        Self::prepare_mapping(default.articles_mapping, &mut self.articles_mapping);
        Self::prepare_mapping(default.content_mapping, &mut self.content_mapping);

        if let Err(error) = if self.enable {
            log::info!("Enabling mouse capture");
            execute!(std::io::stdout(), EnableMouseCapture)
        } else {
            log::info!("Disabling mouse capture");
            execute!(std::io::stdout(), DisableMouseCapture)
        } {
            log::error!("{error}");
        }

        Ok(())
    }

    fn prepare_mapping(
        default: IndexMap<MouseInput, CommandSequence>,
        to: &mut IndexMap<MouseInput, CommandSequence>,
    ) {
        default.into_iter().for_each(|(key_seq, cmd_seq)| {
            to.entry(key_seq).or_insert(cmd_seq);
        });
        to.iter()
            .filter_map(|(mouse_input, command_seq)| {
                command_seq.commands.is_empty().then_some(mouse_input)
            })
            .cloned()
            .collect::<Vec<MouseInput>>()
            .into_iter()
            .for_each(|key| {
                to.shift_remove(&key);
            });
    }
}

impl Default for MouseConfig {
    fn default() -> Self {
        MouseConfig {
            enable: true,
            content_resize: true,
            toggle_tree_on_click: true,
            feeds_mapping: input_mappings![
                "left" => "focus feeds",
                "C-left" => "focus feeds" "toggle",
                "middle" => "toggle",
                "right" => "in feeds read current",
                "scroll_down" => "in feeds down",
                "scroll_up" => "in feeds up",
            ],
            articles_mapping: input_mappings![
                "left" => "focus articles",
                "right" => "in articles read",
                "middle" => "in articles open" "in articles read",
                "C-left" => "flaginvert current",
                "scroll_down" => "in articles down",
                "scroll_up" => "in articles up",
            ],
            content_mapping: input_mappings![
                "left" => "focus content",
                "middle" => "open" "read",
                "right" => "read",
                "scroll_down" => "in content down",
                "scroll_up" => "in content up",
            ],
        }
    }
}
