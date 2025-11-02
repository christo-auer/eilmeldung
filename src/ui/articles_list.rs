use crate::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use news_flash::models::{Article, ArticleID, Feed, FeedID, Marked, Read, Tag, TagID};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
};
use tokio::sync::mpsc::UnboundedSender;

pub struct ArticlesList {
    config: Arc<Config>,

    news_flash_utils: Arc<NewsFlashUtils>,
    message_sender: UnboundedSender<Message>,

    articles: Vec<Article>,
    table: Table<'static>,

    feed_map: HashMap<FeedID, Feed>,
    tags_for_article: HashMap<ArticleID, Vec<TagID>>,
    tag_map: HashMap<TagID, Tag>,

    article_scope: ArticleScope,

    article_search_query: Option<ArticleQuery>,

    table_state: TableState,
    article_filter: Option<AugmentedArticleFilter>,
    is_focused: bool,
}

impl ArticlesList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            article_filter: None,
            article_scope: config.article_scope,
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            articles: Default::default(),
            article_search_query: None,
            tags_for_article: Default::default(),
            tag_map: Default::default(),
            feed_map: Default::default(),
            table_state: Default::default(),
            table: Default::default(),
            is_focused: false,
        }
    }

    async fn build_list(&mut self) -> color_eyre::Result<()> {
        let Some(mut article_filter) = self.article_filter.clone() else {
            return Ok(());
        };

        // get the currently selected article
        let prev_article_id = self.get_current_article_id().cloned();

        {
            let news_flash = self.news_flash_utils.news_flash_lock.read().await;
            // read/unread/marked etc comes from query
            if !article_filter.is_augmented() {
                match self.article_scope {
                    ArticleScope::All => {}
                    ArticleScope::Unread => {
                        article_filter.article_filter.unread = Some(Read::Unread);
                        article_filter.article_filter.marked = None;
                    }
                    ArticleScope::Marked => {
                        article_filter.article_filter.marked = Some(Marked::Marked);
                        article_filter.article_filter.unread = None;
                    }
                }
            }

            article_filter.article_filter.order_by = Some(news_flash::models::OrderBy::Published);
            article_filter.article_filter.order =
                Some(news_flash::models::ArticleOrder::NewestFirst);

            self.articles = news_flash.get_articles(article_filter.article_filter.clone())?;

            let (feeds, _) = news_flash.get_feeds()?;
            self.feed_map = NewsFlashUtils::generate_id_map(&feeds, |f| f.feed_id.clone());

            let (tags, taggings) = news_flash.get_tags()?;
            self.tag_map = NewsFlashUtils::generate_id_map(&tags, |t| t.tag_id.clone());

            self.tags_for_article = NewsFlashUtils::generate_one_to_many(
                &taggings,
                |a| a.article_id.clone(),
                |t| t.tag_id.clone(),
            );

            let position_for_tag = tags
                .iter()
                .enumerate()
                .map(|(pos, tag)| (&tag.tag_id, pos))
                .collect::<HashMap<&TagID, usize>>();

            self.tags_for_article.iter_mut().for_each(|(_, tag_ids)| {
                tag_ids.sort_by(|tag_a, tag_b| {
                    position_for_tag
                        .get(tag_a)
                        .unwrap()
                        .cmp(position_for_tag.get(tag_b).unwrap())
                })
            });

            // apply additional query-based filter
            if article_filter.is_augmented() {
                self.articles = article_filter.article_query.filter(
                    &self.articles,
                    &self.feed_map,
                    &self.tags_for_article,
                    &self.tag_map,
                );
            }
        }

        self.build_table();

        // now, make a sensible choice for selection
        self.restore_sensible_selection(prev_article_id.as_ref())?;

        Ok(())
    }

    fn restore_sensible_selection(
        &mut self,
        article_id: Option<&ArticleID>,
    ) -> color_eyre::Result<()> {
        // save offset distance
        let offset = *self.table_state.offset_mut();
        let offset_distance = self
            .table_state
            .selected()
            .unwrap_or(0)
            .saturating_sub(offset);

        // first, we try to select the article with article_id
        if let Some(article_id) = article_id
            && let Some(index) = self
                .articles
                .iter()
                .position(|article| article.article_id == *article_id)
        {
            *self.table_state.offset_mut() = index.saturating_sub(offset_distance);
            return self.select_index(index);
        }

        // the previous article is not there, next we select the first unread article
        self.table_state.select(Some(0));
        self.select_next_unread()?;

        Ok(())
    }

    fn select_index(&mut self, index: usize) -> color_eyre::Result<()> {
        if let Some(article) = self.articles.get(index) {
            self.table_state.select(Some(index));
            self.message_sender
                .send(Message::Event(Event::ArticleSelected(
                    article.clone(),
                    self.feed_map.get(&article.feed_id).cloned(),
                    self.tags_for_article
                        .get(&article.article_id)
                        .map(|tag_ids| {
                            tag_ids
                                .iter()
                                .filter_map(|tag_id| self.tag_map.get(tag_id))
                                .cloned()
                                .collect()
                        })
                        .clone(),
                )))?;
        }

        self.adjust_offset();
        Ok(())
    }

    fn select_next_unread(&mut self) -> color_eyre::Result<()> {
        let select = self.first_unread().unwrap_or(0);
        self.select_index(select)
    }

    fn first_unread(&self) -> Option<usize> {
        let current_index = self.table_state.selected().unwrap_or(0);

        self.articles
            .iter()
            .enumerate()
            .find(|(index, article)| *index >= current_index && article.unread == Read::Unread)
            .map(|(index, _)| index)
    }

    fn open_in_browser(&self) -> color_eyre::Result<()> {
        if let Some(index) = self.table_state.selected()
            && let Some(article) = self.articles.get(index)
            && let Some(url) = article.url.as_ref()
        {
            webbrowser::open(url.as_ref())?;
        }

        // TODO error handling
        Ok(())
    }

    fn get_current_article_mut(&mut self) -> Option<&mut Article> {
        if let Some(index) = self.table_state.selected() {
            return self.articles.get_mut(index);
        }

        None
    }

    fn get_current_article_id(&self) -> Option<&ArticleID> {
        if let Some(index) = self.table_state.selected() {
            return self.articles.get(index).map(|article| &article.article_id);
        }

        None
    }

    // fn select_by_article_id(&mut self) -> Option<&ArticleID> {
    //     if let Some(index) = self.table_state.selected() {
    //         return self.articles.get(index).map(|article| &article.article_id);
    //     }
    //
    //     None
    // }

    async fn set_all_read_status(&mut self, read: Read) -> color_eyre::Result<()> {
        let article_ids: Vec<ArticleID> = self
            .articles
            .iter()
            .map(|article| article.article_id.clone())
            .collect();

        self.news_flash_utils.set_article_status(article_ids, read);

        self.articles
            .iter_mut()
            .for_each(|article| article.unread = read);

        Ok(())
    }

    async fn set_current_read_status(&mut self, read: Option<Read>) -> color_eyre::Result<()> {
        let news_flash_utils = self.news_flash_utils.clone();
        if let Some(article) = self.get_current_article_mut() {
            let new_state = match read {
                Some(state) => state,
                None => article.unread.invert(),
            };

            let article_id = article.article_id.clone();

            news_flash_utils.set_article_status(vec![article_id], new_state);

            // update locally
            article.unread = new_state;
        }

        Ok(())
    }

    fn build_scope_title(&self) -> String {
        if let Some(article_filter) = self.article_filter.clone()
            && article_filter.is_augmented()
        {
            return "".to_string();
        }

        let to_icon = |scope: ArticleScope| -> char {
            if scope == self.article_scope {
                '󰐾'
            } else {
                ''
            }
        };

        format!(
            "{} All {} Unread {} Marked",
            to_icon(ArticleScope::All),
            to_icon(ArticleScope::Unread),
            to_icon(ArticleScope::Marked),
        )
    }

    fn adjust_offset(&mut self) {
        let Some(index) = self.table_state.selected() else {
            return;
        };
        let offset = self.table_state.offset_mut();
        let max_lines_above = self.config.theme.articles_list_height_lines as usize
            - (self.config.articles_list_visible_articles_after_selection + 1);

        if index.saturating_sub(*offset) > max_lines_above {
            *offset = index.saturating_sub(max_lines_above);
        }
    }

    fn search(
        &self,
        articles: &[Article],
        article_query: &ArticleQuery,
        reversed: bool,
    ) -> Option<usize> {
        let predicate = |article: &Article| {
            article_query.test(
                article,
                &self.feed_map,
                &self.tags_for_article,
                &self.tag_map,
            )
        };

        if !reversed {
            articles.iter().position(predicate)
        } else {
            articles.iter().rposition(predicate)
        }
    }

    fn search_next(&mut self, skip_current: bool, reversed: bool) -> color_eyre::Result<()> {
        let offset = if skip_current { 1 } else { 0 };
        let Some(article_query) = self.article_search_query.as_ref() else {
            return Err(color_eyre::eyre::eyre!("no search query"));
        };

        if let Some(selected) = self.table_state.selected() {
            let split_index = if !reversed {
                selected + offset
            } else {
                selected.saturating_sub(offset)
            };

            let slices = self.articles.split_at(split_index);

            let (first_range, second_range) = if reversed {
                slices
            } else {
                (slices.1, slices.0)
            };

            let (first_offset, second_offset) = if !reversed {
                (split_index, 0)
            } else {
                (0, split_index)
            };

            match self.search(first_range, article_query, reversed) {
                Some(index) => {
                    return self.select_index(index + first_offset);
                }
                None => match self.search(second_range, article_query, reversed) {
                    Some(index) => {
                        tooltip(
                            &self.message_sender,
                            if !reversed {
                                "end reached, starting from beginning"
                            } else {
                                "beginning reached, starting from end"
                            },
                            TooltipFlavor::Warning,
                        )?;
                        return self.select_index(index + second_offset);
                    }
                    None => {
                        tooltip(
                            &self.message_sender,
                            "no match found",
                            TooltipFlavor::Warning,
                        )?;
                    }
                },
            }
        }
        {}

        Ok(())
    }

    fn build_table(&mut self) {
        let selected_style = if self.is_focused {
            Style::new().reversed()
        } else {
            Style::new().underlined()
        };

        let read_icon = self.config.read_icon.to_string();
        let unread_icon = self.config.unread_icon.to_string();
        let marked_icon = self.config.marked_icon.to_string();
        let unmarked_icon = self.config.unmarked_icon.to_string();
        let placeholders: Vec<&str> = self
            .config
            .article_table
            .split(",")
            .map(|placeholder| placeholder.trim())
            .collect();

        let mut max_tags: u16 = 0;

        let entries: Vec<Row> = self
            .articles
            .iter()
            .map(|article| {
                let row_vec: Vec<Line> = placeholders
                    .iter()
                    .map(|placeholder| match *placeholder {
                        "{title}" => article
                            .title
                            .clone()
                            .unwrap_or("?".into())
                            .to_string()
                            .into(),
                        "{tag_icons}" => {
                            Line::from(match self.tags_for_article.get(&article.article_id) {
                                Some(tag_ids) => {
                                    max_tags = u16::max(max_tags, tag_ids.len() as u16);

                                    tag_ids
                                        .iter()
                                        .map(|tag_id| {
                                            let Some(tag) = self.tag_map.get(tag_id) else {
                                                return Span::from("");
                                            };

                                            let style = match NewsFlashUtils::tag_color(tag) {
                                                Some(color) => self.config.theme.article.fg(color),
                                                None => self.config.theme.article,
                                            };
                                            Span::styled(self.config.tag_icon.to_string(), style)
                                        })
                                        .collect::<Vec<Span>>()
                                }
                                None => vec![Span::from("")],
                            })
                        }
                        "{author}" => article
                            .author
                            .clone()
                            .unwrap_or("?".into())
                            .to_string()
                            .into(),
                        "{feed}" => self
                            .feed_map
                            .get(&article.feed_id)
                            .map(|feed| feed.label.clone())
                            .unwrap_or("?".into())
                            .to_string()
                            .into(),
                        "{date}" => article
                            .date
                            .with_timezone(&chrono::Local)
                            .format(&self.config.date_format)
                            .to_string()
                            .into(),
                        "{age}" => {
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(article.date);

                            let weeks = duration.num_weeks();
                            let days = duration.num_days();
                            let hours = duration.num_hours();
                            let minutes = duration.num_minutes();
                            let seconds = duration.num_seconds();

                            if weeks > 0 {
                                format!("{:>2}w", weeks)
                            } else if days > 0 {
                                format!("{:>2}d", days)
                            } else if hours > 0 {
                                format!("{:>2}h  ", hours)
                            } else if minutes > 0 {
                                format!("{:>2}m", minutes)
                            } else {
                                format!("{:>2}s", seconds)
                            }
                        }
                        .into(),
                        "{read}" => if article.unread == Read::Read {
                            format!(" {} ", read_icon)
                        } else {
                            format!(" {} ", unread_icon)
                        }
                        .into(),
                        "{marked}" => if article.marked == Marked::Marked {
                            format!(" {} ", marked_icon)
                        } else {
                            format!(" {} ", unmarked_icon)
                        }
                        .into(),
                        "{url}" => article
                            .url
                            .clone()
                            .map(|url| url.to_string())
                            .unwrap_or("?".into())
                            .into(),
                        _ => format!("{placeholder}?").into(),
                    })
                    .collect();

                let style = match self.article_search_query.as_ref() {
                    Some(query)
                        if query.test(
                            article,
                            &self.feed_map,
                            &self.tags_for_article,
                            &self.tag_map,
                        ) =>
                    {
                        info!("article MATCHES");
                        self.config.theme.article_highlighted
                    }
                    _ => self.config.theme.article,
                };

                Row::new(row_vec).style(style)
            })
            .collect();

        let constraint_for_placeholder = |placeholder: &str| {
            if placeholder == "{read}" || placeholder == "{marked}" {
                Constraint::Length(3)
            } else if placeholder == "{age}" {
                Constraint::Length(4)
            } else if placeholder == "{date}" {
                Constraint::Length(self.config.date_format.len() as u16)
            } else if placeholder == "{tag_icons}" {
                Constraint::Length(max_tags)
            } else {
                Constraint::Min(1)
            }
        };

        self.table = Table::new(
            entries,
            placeholders
                .iter()
                .map(|placeholder| constraint_for_placeholder(placeholder))
                .collect::<Vec<Constraint>>(),
        )
        .row_highlight_style(selected_style)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .title_top(self.build_scope_title())
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(if self.is_focused {
                    self.config.theme.focused_border_style
                } else {
                    self.config.theme.border_style
                }),
        );
    }
}

impl Widget for &mut ArticlesList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        StatefulWidget::render(&self.table, area, buf, &mut self.table_state);
    }
}

impl crate::messages::MessageReceiver for ArticlesList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        let prev_selected_index = self.table_state.selected();
        let mut now_selected_index: Option<usize> = None;

        match message {
            Message::Command(NavigateUp) if self.is_focused => self.table_state.select_previous(),
            Message::Command(NavigateDown) if self.is_focused => self.table_state.select_next(),
            Message::Command(NavigatePageUp) if self.is_focused => self
                .table_state
                .scroll_up_by(self.config.theme.articles_list_height_lines - 1),
            Message::Command(NavigatePageDown) if self.is_focused => {
                self.table_state.scroll_down_by(
                    self.config.theme.articles_list_height_lines
                        - self.config.articles_list_visible_articles_after_selection as u16,
                )
            }
            Message::Command(NavigateFirst) if self.is_focused => self.table_state.select_first(),
            Message::Command(NavigateLast) if self.is_focused => {
                self.table_state.select_last();
                // manually "select" as select_last does not know the number of rows
                now_selected_index = Some(self.articles.len() - 1);
            }

            Message::Event(AsyncOperationFailed(_, _)) => {
                self.build_list().await?;
                self.select_next_unread()?;
            }

            Message::Event(ArticlesSelected(augmented_article_filter)) => {
                self.article_filter = Some(augmented_article_filter.clone());
                self.build_list().await?;
                self.table_state.select(Some(0));
                self.select_next_unread()?;
            }

            Message::Command(ArticleListSetScope(scope)) => {
                self.article_scope = *scope;
                self.build_list().await?;
                self.select_next_unread()?;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::ArticleSelection;
                self.build_list().await?;
            }

            Message::Command(ArticleCurrentOpenInBrowser) => {
                self.open_in_browser()?;
            }

            Message::Command(ArticleCurrentSetRead) => {
                self.set_current_read_status(Some(Read::Read)).await?;
                self.build_table();
            }

            Message::Command(ArticleCurrentSetUnread) => {
                self.set_current_read_status(Some(Read::Unread)).await?;
                self.build_table();
            }

            Message::Command(ArticleCurrentToggleRead) => {
                self.set_current_read_status(None).await?;
                self.build_table();
            }

            Message::Command(ArticleListSetAllRead) => {
                self.set_all_read_status(Read::Read).await?;
                self.build_table();
            }

            Message::Command(ArticleListSetAllUnread) => {
                self.set_all_read_status(Read::Unread).await?;
                self.build_table();
            }

            Message::Event(AsyncMarkArticlesAsReadFinished) => {
                self.build_list().await?;
            }

            Message::Command(ArticleListSelectNextUnread) => {
                self.select_next_unread()?;
            }

            Message::Command(ArticleListSearch(query)) => {
                self.article_search_query = Some(query.clone());
                self.build_table();
                self.search_next(false, false)?;
            }

            Message::Command(ArticleListSearchNext) => {
                self.search_next(true, false)?;
            }

            Message::Command(ArticleListSearchPrevious) => {
                self.search_next(true, true)?;
            }
            _ => {}
        }

        if now_selected_index.is_none() {
            now_selected_index = self.table_state.selected();
        }
        if prev_selected_index != now_selected_index {
            log::trace!("selecting {:?}", now_selected_index);
            match now_selected_index {
                Some(index) => {
                    self.select_index(index)?;
                }
                None => {
                    self.select_next_unread()?;
                }
            }
        }

        Ok(())
    }
}
