use crate::prelude::*;

use std::{str::FromStr, sync::Arc};

use ratatui::{
    crossterm::event::KeyCode,
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, Borders},
};
use strum::{EnumMessage, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

pub struct CommandInput {
    config: Arc<Config>,

    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    text_input: TextArea<'static>,
    command_hint: Option<Line<'static>>,
    completion_targets: Option<Vec<String>>,
    completion_prefix: String,
    help_dialog_open: bool,

    history: Vec<String>,
    history_index: usize,

    is_active: bool,
}

impl CommandInput {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            text_input: TextArea::default(),
            history: Vec::default(),
            completion_targets: None,
            completion_prefix: String::default(),
            help_dialog_open: false,
            command_hint: None,
            history_index: 0,
            is_active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    fn to_command(&self) -> Result<Command, CommandParseError> {
        let input = self.text_input.lines()[0].as_str();

        Command::parse(input, false)
    }

    fn on_submit(&mut self) -> color_eyre::Result<()> {
        match self.to_command() {
            Ok(command) => {
                self.is_active = false;
                self.message_sender
                    .send(Message::Command(command.clone()))?;
                self.update_current_history_entry();
            }
            Err(err) => {
                self.message_sender
                    .send(Message::Event(Event::Tooltip(Tooltip::from_str(
                        err.to_string().as_str(),
                        TooltipFlavor::Error,
                    ))))?;
                // handle error
            }
        };
        Ok(())
    }

    fn clear(&mut self, s: &str) {
        self.text_input.select_all();
        self.text_input.delete_char();
        self.text_input.insert_str(s);
    }

    fn on_history(&mut self, index: usize) {
        self.history_index = index;
        let history_entry = self.history.get(index).unwrap().to_string();
        self.clear(&history_entry);
    }

    fn on_history_previous(&mut self) {
        if let Some(index) = self.history[0..self.history_index]
            .iter()
            .rposition(|entry| {
                entry.starts_with(self.history.last().map(String::as_str).unwrap_or_default())
            })
        {
            self.on_history(index);
        }
    }

    fn on_history_next(&mut self) {
        if let Some(index) = self.history[self.history_index + 1..]
            .iter()
            .position(|entry| {
                entry.starts_with(self.history.last().map(String::as_str).unwrap_or_default())
            })
        {
            self.on_history(index + self.history_index + 1);
        }
    }

    fn get_current_input_to_cursor(&self) -> String {
        let col = self.text_input.cursor().1;
        string_manipulation_utf8::substring(&self.text_input.lines()[0], 0, col as isize)
    }

    fn get_current_word(&self) -> (String, String) {
        let current_input = self.get_current_input_to_cursor();
        match current_input.rsplit_once(|c: char| c.is_whitespace()) {
            None => ("".to_owned(), current_input),
            Some(split) => (split.0.to_owned(), split.1.to_owned()),
        }
    }

    fn get_first_word(&self) -> String {
        let current_input = self.get_current_input_to_cursor();
        match current_input.split_once(|c: char| c.is_whitespace()) {
            None => "".to_owned(),
            Some((word, _)) => word.to_owned(),
        }
    }

    async fn update_command_help(&mut self) -> color_eyre::Result<()> {
        let (partial_command, current_part) = self.get_current_word();
        let current_part = current_part.to_owned();

        let parse_result = Command::parse(&partial_command, true);

        use CommandParseError as E;
        match parse_result {
            Err(E::ArticleScopeExpected) => {
                self.generate_help_content_article_scope(&current_part)
                    .await?
            }
            Err(E::TagExpected) => self.generate_help_content_for_tag(&current_part).await?,
            Err(E::TargetOrArticleScopeExpected) => {
                self.generate_help_content_target_or_article_scope(&current_part)?
            }
            Err(E::ArticleQueryExpected(..)) => {
                self.generate_help_content_article_query(&current_part)?
            }
            Err(E::URLExpected(..)) => self.generate_help_content_url(&current_part)?,
            Err(E::CommandNameExpected(..)) => {
                self.generate_help_content_command_name(&current_part)?
            }
            Err(E::CommandExpected) => self.generate_help_content_command(&current_part)?,
            Err(E::PanelExpected) => self.generate_help_content_panel(&current_part)?,
            Err(E::ColorExpected(..)) => self.generate_help_content_color(&current_part)?,

            Err(E::ShareTargetExpected) => {
                self.generate_help_content_share_target(&current_part)?
            }

            Err(E::FilePathExpected) => self.generate_help_content_file_path(&current_part)?,

            Err(other_err) => self.generate_help_content_error(other_err, &current_part)?,
            Ok(command) => self.generate_help_content_complete_command(command, &current_part)?,
        }
        Ok(())
    }

    fn on_complete(&mut self, forward: bool) {
        let Some(completion_targets) = self.completion_targets.as_ref() else {
            return;
        };

        if completion_targets.is_empty() {
            return;
        }

        let completion_targets = completion_targets
            .iter()
            .filter(|target| target.starts_with(&self.completion_prefix))
            .collect::<Vec<&String>>();

        let (_, current_word) = self.get_current_word();

        let num_targets = completion_targets.len();
        let completion = match completion_targets
            .iter()
            .position(|target| *target == &current_word)
        {
            Some(index) => {
                if forward {
                    Some(completion_targets[(index + 1) % num_targets].clone())
                } else {
                    Some(completion_targets[(index + num_targets - 1) % num_targets].clone())
                }
            }
            None => Some(
                completion_targets
                    .first()
                    .map(|s| (*s).to_owned())
                    .unwrap_or_else(|| current_word.to_owned()),
            ),
        };

        if let Some(completion) = completion {
            if !current_word.is_empty() {
                self.text_input.delete_word();
            }
            self.text_input.insert_str(completion);
        }
    }

    fn distribute_in_columns<'a>(items: Vec<Line<'a>>, columns: usize) -> Text<'a> {
        let max_widths = items
            .iter()
            .map(|line| line.width())
            .collect::<Vec<usize>>()
            .chunks(columns)
            .fold(vec![0; columns], |max_widths, line_lengths| {
                let mut line_lengths = line_lengths.to_owned();
                let mut fill: Vec<usize> = vec![0; columns - line_lengths.len()];
                line_lengths.append(&mut fill);

                line_lengths
                    .into_iter()
                    .zip(max_widths)
                    .map(|(a, b)| usize::max(a, b))
                    .collect::<Vec<usize>>()
            });

        let column_lines = items
            .into_iter()
            .enumerate()
            .map(|(i, mut line)| {
                let fill = (max_widths[i % max_widths.len()]).saturating_sub(line.width());
                line.push_span(" ".repeat(fill + 1));
                line
            })
            .collect::<Vec<Line<'a>>>()
            .chunks(columns)
            .map(|chunk| {
                Line::from(
                    chunk
                        .iter()
                        .flat_map(|line| line.spans.iter().cloned())
                        .collect::<Vec<Span<'_>>>(),
                )
            })
            .collect::<Vec<Line<'_>>>();

        Text::from(column_lines)
    }

    async fn generate_help_content_for_tag(
        &mut self,
        current_part: &str,
    ) -> color_eyre::Result<()> {
        let (tags, _) = {
            let news_flash = self.news_flash_utils.news_flash_lock.read().await;
            news_flash.get_tags()?
        };

        self.completion_targets = Some(tags.iter().map(|tag| tag.label.to_owned()).collect());

        let tag_spans = tags
            .into_iter()
            .map(|tag| {
                NewsFlashUtils::tag_to_line(
                    &tag,
                    &self.config,
                    (!tag.label.starts_with(current_part))
                        .then_some(self.config.theme.inactive().fg.unwrap_or_default()),
                )
            })
            .collect::<Vec<Line<'_>>>();

        let text = Self::distribute_in_columns(tag_spans, 4);

        self.show_help_dialog("Tags".to_owned(), text)?;

        Ok(())
    }

    fn show_help_dialog(
        &mut self,
        title: String,
        paragraph: Text<'static>,
    ) -> Result<(), color_eyre::eyre::Error> {
        self.message_sender
            .send(Message::Event(Event::ShowHelpPopup(title, paragraph)))?;
        self.help_dialog_open = true;
        Ok(())
    }

    fn hide_help_dialog(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.message_sender
            .send(Message::Event(Event::HideHelpPopup))?;
        self.help_dialog_open = false;
        Ok(())
    }

    fn generate_help_tab_std<'a, T, I>(
        &self,
        items: I,
        current: &str,
    ) -> impl Iterator<Item = Line<'a>>
    where
        T: AsRef<str> + strum::EnumMessage + 'a,
        I: IntoIterator<Item = T>,
    {
        self.generate_help_tab(
            items,
            current,
            |target| target.as_ref().to_owned(),
            |target| target.get_detailed_message().unwrap_or("?").to_owned(),
        )
    }

    fn generate_help_tab<'a, T, I>(
        &self,
        items: I,
        current: &str,
        to_key: impl Fn(&T) -> String,
        to_value: impl Fn(&T) -> String,
    ) -> impl Iterator<Item = Line<'a>>
    where
        T: 'a,
        I: IntoIterator<Item = T>,
    {
        let current = current.to_owned();
        items.into_iter().flat_map(move |enum_message| {
            let id = to_key(&enum_message);
            let modifier = if id == current.as_str() {
                Modifier::REVERSED
            } else {
                Modifier::default()
            };
            vec![
                Line::from(Span::styled(
                    id,
                    self.config.theme.header().add_modifier(modifier),
                )),
                Line::from(Span::styled(
                    to_value(&enum_message),
                    self.config.theme.paragraph(),
                )),
            ]
        })
    }

    fn generate_help_content_enum<T>(
        &mut self,
        current_part: &str,
        title: &str,
        to_complete: impl Fn(&T) -> Option<String>,
        to_key: impl Fn(&T) -> String,
        to_value: impl Fn(&T) -> String,
    ) -> color_eyre::Result<()>
    where
        T: IntoEnumIterator + 'static,
    {
        let text = Self::distribute_in_columns(
            self.generate_help_tab(T::iter(), current_part, to_key, to_value)
                .collect::<Vec<Line<'static>>>(),
            2,
        );

        self.show_help_dialog(title.to_owned(), text)?;

        let targets = T::iter()
            .filter_map(|scope| to_complete(&scope))
            .collect::<Vec<String>>();

        self.completion_targets = Some(targets);
        Ok(())
    }

    fn generate_help_content_target_or_article_scope(
        &mut self,
        current_part: &str,
    ) -> color_eyre::Result<()> {
        let text = Self::distribute_in_columns(
            self.generate_help_tab_std(ActionTarget::iter(), current_part)
                .chain(self.generate_help_tab_std(ActionScope::iter(), current_part))
                .collect::<Vec<Line<'_>>>(),
            2,
        );

        self.show_help_dialog("Action or Article Scope".to_owned(), text)?;

        let targets = ActionTarget::iter()
            .map(|target| target.get_message().unwrap_or("?").to_owned())
            .chain(ActionScope::iter().map(|scope| scope.get_message().unwrap_or("?").to_owned()))
            .collect::<Vec<String>>();

        self.completion_targets = Some(targets);

        Ok(())
    }

    fn generate_help_content_article_query(
        &mut self,
        current_part: &str,
    ) -> color_eyre::Result<()> {
        let text = Self::distribute_in_columns(
            QueryToken::iter()
                .filter(|token| token.get_message().is_some())
                .flat_map(|token| {
                    let modifier = if token.as_ref() == current_part {
                        Modifier::REVERSED
                    } else {
                        Modifier::default()
                    };
                    vec![
                        Line::from(Span::styled(
                            token.as_ref().to_owned(),
                            self.config.theme.header().add_modifier(modifier),
                        )),
                        Line::from(Span::styled(
                            token.get_message().unwrap_or_default().to_owned(),
                            self.config.theme.paragraph(),
                        )),
                        Line::from(Span::styled(
                            token.get_detailed_message().unwrap_or_default().to_owned(),
                            self.config.theme.paragraph(),
                        )),
                    ]
                })
                .collect::<Vec<Line<'_>>>(),
            3,
        );

        self.show_help_dialog("Query".to_owned(), text)?;

        self.completion_targets = Some(
            QueryToken::iter()
                .filter(|token| token.get_message().is_some())
                .map(|token| token.as_ref().to_owned())
                .collect(),
        );

        Ok(())
    }

    fn generate_help_content_url(&mut self, _current_part: &str) -> color_eyre::Result<()> {
        let line = Text::from(Line::from(vec![
            Span::styled("URL expected ".to_owned(), self.config.theme.header()),
            Span::styled(
                "e.g., https://www.feedprovider.com/rss".to_owned(),
                self.config.theme.paragraph(),
            ),
        ]));
        self.show_help_dialog("URL".to_owned(), line)?;
        Ok(())
    }

    fn generate_help_content_command_name(&mut self, current_part: &str) -> color_eyre::Result<()> {
        let commands = Command::iter()
            .filter(|command| {
                command
                    .as_ref()
                    .starts_with(self.completion_prefix.as_str())
            })
            .collect::<Vec<Command>>();

        let text = if commands.is_empty() {
            Text::styled("no matches", self.config.theme.header())
        } else if commands.len() < 15 {
            Self::distribute_in_columns(
                self.generate_help_tab_std(commands, current_part)
                    .collect::<Vec<Line<'_>>>(),
                2,
            )
        } else {
            Self::distribute_in_columns(
                commands
                    .iter()
                    .map(|command| {
                        let id = command.as_ref().to_owned();
                        let modifier = if id == current_part {
                            Modifier::REVERSED
                        } else {
                            Modifier::default()
                        };
                        Line::from(Span::styled(
                            id,
                            self.config.theme.header().add_modifier(modifier),
                        ))
                    })
                    .collect(),
                5,
            )
        };

        let targets = Command::iter()
            .map(|target| target.as_ref().to_owned())
            .collect::<Vec<String>>();

        self.completion_targets = Some(targets);

        self.show_help_dialog("Command".to_owned(), text)?;

        Ok(())
    }

    fn generate_help_content_command(&mut self, _current_part: &str) -> color_eyre::Result<()> {
        self.hide_help_dialog()?;
        self.completion_targets = None;
        Ok(())
    }

    fn generate_help_content_color(&mut self, current_part: &str) -> color_eyre::Result<()> {
        use Color as C;
        let color = [
            C::Black,
            C::Red,
            C::Green,
            C::Yellow,
            C::Blue,
            C::Magenta,
            C::Cyan,
            C::Gray,
            C::DarkGray,
            C::LightRed,
            C::LightGreen,
            C::LightYellow,
            C::LightBlue,
            C::LightMagenta,
            C::LightCyan,
            C::White,
            C::Rgb(128, 128, 128),
            // C::Indexed(u8),
        ];

        let color_lines = color
            .iter()
            .map(|color| {
                let color_name = match color {
                    C::Rgb(..) => "#FFFFFF".to_owned(),
                    color => color.to_string().to_lowercase(),
                };

                let modifier = if current_part == color_name.as_str() {
                    Modifier::REVERSED
                } else {
                    Modifier::default()
                };

                Line::from(vec![
                    Span::styled("#abc", Style::default().fg(*color).reversed()),
                    Span::raw(" "),
                    Span::styled(
                        color_name,
                        self.config.theme.header().add_modifier(modifier),
                    ),
                ])
            })
            .collect::<Vec<Line<'_>>>();

        let columns = Self::distribute_in_columns(color_lines, 4);

        self.show_help_dialog("Color".to_owned(), columns)?;

        self.completion_targets = Some(
            color
                .iter()
                .map(|color| color.to_string().to_lowercase())
                .collect(),
        );

        Ok(())
    }

    fn generate_help_content_share_target(&mut self, current_part: &str) -> color_eyre::Result<()> {
        let text = Self::distribute_in_columns(
            self.generate_help_tab_std(self.config.share_targets.clone(), current_part)
                .collect::<Vec<Line<'_>>>(),
            2,
        );

        let targets = self
            .config
            .share_targets
            .iter()
            .map(|target| match target {
                ShareTarget::Custom(name, ..) => name.to_owned(),
                target => target.as_ref().to_owned(),
            })
            .collect::<Vec<String>>();
        self.completion_targets = Some(targets);

        self.show_help_dialog("Share Targets".to_owned(), text)?;

        Ok(())
    }

    fn generate_help_content_error(
        &mut self,
        _other_err: CommandParseError,
        _current_part: &str,
    ) -> color_eyre::Result<()> {
        self.hide_help_dialog()?;
        self.completion_targets = None;
        Ok(())
    }

    fn generate_help_content_complete_command(
        &mut self,
        _command: Command,
        _current_part: &str,
    ) -> color_eyre::Result<()> {
        self.hide_help_dialog()?;
        self.completion_targets = None;
        Ok(())
    }

    fn generate_help_content_panel(&mut self, current_part: &str) -> color_eyre::Result<()> {
        self.generate_help_content_enum::<Panel>(
            current_part,
            "Panel",
            |panel| Some(panel.as_ref().to_owned()),
            |panel| panel.get_message().unwrap_or_default().to_owned(),
            |panel| panel.get_detailed_message().unwrap_or_default().to_owned(),
        )
    }

    async fn generate_help_content_article_scope(
        &mut self,
        current_part: &str,
    ) -> color_eyre::Result<()> {
        self.generate_help_content_enum::<ArticleScope>(
            current_part,
            "Article Scope",
            |scope| Some(scope.as_ref().to_owned()),
            |scope| scope.get_message().unwrap_or_default().to_owned(),
            |scope| scope.get_detailed_message().unwrap_or_default().to_owned(),
        )
    }

    fn update_completion_prefix(&mut self) {
        let (_, prefix) = self.get_current_word();
        self.completion_prefix = prefix.to_owned();
    }

    fn update_current_history_entry(&mut self) {
        self.history_index = self.history.len() - 1;
        *self.history.last_mut().unwrap() = self.text_input.lines()[0].to_string();
    }

    fn update_command_hint(&mut self) {
        let current_input = self.text_input.lines()[0].as_str();
        if let Ok(command) = Command::parse(current_input, false) {
            self.command_hint = Some(Line::from(vec![
                Span::styled(command.to_string(), self.config.theme.header()),
                Span::styled(
                    format!(" ({})", command.get_message().unwrap_or_default()),
                    self.config.theme.paragraph(),
                ),
            ]));
            return;
        }

        if let Ok(command) = Command::from_str(&self.get_first_word()) {
            self.command_hint = Some(Line::from(vec![
                Span::styled(
                    command.get_message().unwrap_or(command.as_ref()).to_owned(),
                    self.config.theme.header(),
                ),
                Span::styled(
                    format!("  {}", command.get_detailed_message().unwrap_or_default()),
                    self.config.theme.paragraph(),
                ),
            ]));
            return;
        }

        self.command_hint = Some(Line::from(Span::styled(
            "Press <TAB> for help.",
            self.config.theme.paragraph(),
        )));
    }

    fn generate_help_content_file_path(&mut self, _current_part: &str) -> color_eyre::Result<()> {
        self.hide_help_dialog()?;
        self.completion_targets = None;
        Ok(())
    }
}

impl Widget for &mut CommandInput {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(if self.is_active {
                self.config.theme.border_focused()
            } else {
                self.config.theme.border()
            })
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);

        let [command_input_chunk, command_hint_chunk] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
                .horizontal_margin(1)
                .areas(inner_area);

        let [preset_command_chunk, text_input_chunk] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(layout::Flex::Start)
            .spacing(1)
            .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
            .areas(command_input_chunk);
        self.text_input.set_style(self.config.theme.command_input());

        block.render(area, buf);
        Text::from(self.config.command_line_prompt_icon.to_string())
            .style(self.config.theme.header())
            .render(preset_command_chunk, buf);
        self.text_input.render(text_input_chunk, buf);

        let [_, command_hint_chunk] =
            Layout::horizontal([Constraint::Length(2), Constraint::Min(1)])
                .flex(Flex::Start)
                .areas(command_hint_chunk);

        self.command_hint
            .clone()
            .unwrap_or_default()
            .render(command_hint_chunk, buf);
    }
}

impl crate::messages::MessageReceiver for CommandInput {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        let mut view_needs_update = true;
        match message {
            Message::Event(Event::Key(key_event)) if self.is_active => {
                let key: Key = (*key_event).into();

                match (
                    key,
                    self.config
                        .input_config
                        .match_single_key_to_single_command(&key),
                ) {
                    (_, Some(Command::InputAbort)) => {
                        if self.help_dialog_open {
                            self.hide_help_dialog()?;
                        } else {
                            self.history.remove(self.history.len() - 1);
                            self.is_active = false;
                        }
                    }
                    (_, Some(Command::InputSubmit)) => {
                        if self.help_dialog_open {
                            self.hide_help_dialog()?;
                        }
                        self.on_submit()?;
                    }

                    (_, Some(Command::InputClear)) => self.clear(""),

                    (Key::Just(KeyCode::Down), _) => self.on_history_next(),
                    (Key::Just(KeyCode::Up), _) => self.on_history_previous(),
                    (Key::Just(KeyCode::Tab), _) => {
                        self.update_command_help().await?;
                        self.on_complete(true);
                    }
                    (Key::Just(KeyCode::BackTab), _) => {
                        self.update_command_help().await?;
                        self.on_complete(false);
                    }
                    _ => {
                        if self.text_input.input(*key_event) {
                            self.update_completion_prefix();
                            self.update_current_history_entry();
                        }
                    }
                }
                if self.help_dialog_open {
                    self.update_command_help().await?;
                }

                self.update_command_hint();
            }

            Message::Command(Command::CommandLineOpen(preset_command)) => {
                self.is_active = true;
                self.clear("");

                let preset_command = preset_command
                    .clone()
                    .map(|mut preset_command| {
                        preset_command.push(' ');
                        preset_command
                    })
                    .unwrap_or_default();

                self.history.push(preset_command.to_string());
                self.history_index = self.history.len() - 1;
                self.completion_prefix = preset_command.to_owned();
                self.text_input.insert_str(preset_command);
                self.completion_targets = None;
                self.update_command_hint();
            }

            _ => {
                view_needs_update = false;
            }
        }

        if view_needs_update {
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        Ok(())
    }
}
