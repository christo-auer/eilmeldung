use ratatui::text::Text;

use crate::config::Config;

#[derive(Clone, Debug)]
pub enum TooltipFlavor {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Tooltip {
    pub contents: String,
    pub flavor: TooltipFlavor,
}

impl Tooltip {
    pub fn new(contents: String, flavor: TooltipFlavor) -> Self {
        Self { contents, flavor }
    }

    pub fn to_text<'a>(&self, config: &Config) -> Text<'a> {
        let style = match self.flavor {
            TooltipFlavor::Info => config.theme.tooltip_info,
            TooltipFlavor::Warning => config.theme.tooltip_warning,
            TooltipFlavor::Error => config.theme.tooltip_error,
        };
        Text::from(self.contents.clone()).style(style)
    }
}
