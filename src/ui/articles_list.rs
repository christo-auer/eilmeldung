use std::collections::HashMap;
use std::sync::Arc;

use news_flash::models::{
    Article, ArticleFilter, ArticleID, Feed, FeedID, Marked, Read, Tag, TagID,
};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::query::AugmentedArticleFilter;
use crate::{
    app::AppState,
    commands::{Command, Event, Message, MessageReceiver},
    config::Config,
    newsflash_utils::NewsFlashUtils,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ArticleScope {
    All,
    Unread,
    Marked,
}

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
            article_scope: config.clone().article_scope,
            news_flash_utils: news_flash_utils.clone(),
            message_sender,
            articles: Default::default(),
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

        {
            let news_flash = self.news_flash_utils.news_flash_lock.read().await;

            match self.article_scope {
                ArticleScope::All => {}
                ArticleScope::Unread => {
                    article_filter.article_filter.unread = Some(Read::Unread);
                }
                ArticleScope::Marked => {
                    article_filter.article_filter.marked = Some(Marked::Marked);
                }
            }

            article_filter.article_filter.order_by = Some(news_flash::models::OrderBy::Published);
            article_filter.article_filter.order =
                Some(news_flash::models::ArticleOrder::NewestFirst);

            self.articles = news_flash.get_articles(article_filter.article_filter)?;

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

            let article_filter = self.article_filter.as_ref().unwrap();
            if article_filter.is_augmented() {
                self.articles = article_filter.filter(
                    &self.articles,
                    &self.feed_map,
                    &self.tags_for_article,
                    &self.tag_map,
                );
            }
        }

        Ok(())
    }

    fn select_index(&mut self, index: usize) -> color_eyre::Result<()> {
        if let Some(article) = self.articles.get(index) {
            self.table_state.select(Some(index));
            self.message_sender
                .send(Message::Event(Event::ArticleSelected(article.clone())))?;
        }
        Ok(())
    }

    fn select_first_unread(&mut self) -> color_eyre::Result<()> {
        let select = self.first_unread().unwrap_or(0);
        self.select_index(select)
    }

    fn first_unread(&self) -> Option<usize> {
        self.articles
            .iter()
            .enumerate()
            .find(|(_, article)| article.unread == Read::Unread)
            .map(|(index, _)| index)
    }

    fn open_in_browser(&self) -> color_eyre::Result<()> {
        if let Some(index) = self.table_state.selected()
            && let Some(article) = self.articles.get(index)
            && let Some(url) = article.url.clone()
        {
            webbrowser::open(url.to_string().as_str())?;
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

            article.unread = new_state;
        }

        Ok(())
    }

    fn build_scope_title(&self) -> String {
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
}

impl Widget for &mut ArticlesList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = if self.is_focused {
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

                Row::new(row_vec)
            })
            .collect();

        let constraint_for_placeholder = |placeholder: &str| {
            if placeholder == "{read}" || placeholder == "{marked}" {
                Constraint::Length(3)
            } else if placeholder == "{age}" {
                Constraint::Length(3)
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
        .style(self.config.theme.article)
        .row_highlight_style(highlight_style)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .title_top(self.build_scope_title())
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(self.config.theme.border_style),
        );

        StatefulWidget::render(&self.table, area, buf, &mut self.table_state);
    }
}

impl MessageReceiver for ArticlesList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        use Command::*;
        use Event::*;

        let selected_before = self.table_state.selected();

        match message {
            Message::Command(NavigateUp) if self.is_focused => self.table_state.select_previous(),
            Message::Command(NavigateDown) if self.is_focused => self.table_state.select_next(),
            Message::Command(NavigatePageUp) if self.is_focused => self
                .table_state
                .scroll_up_by(self.config.input_config.scroll_amount as u16),
            Message::Command(NavigatePageDown) if self.is_focused => self
                .table_state
                .scroll_down_by(self.config.input_config.scroll_amount as u16),
            Message::Command(NavigateFirst) if self.is_focused => self.table_state.select_first(),
            Message::Command(NavigateLast) if self.is_focused => self.table_state.select_last(),

            Message::Event(AsyncOperationFailed(_)) => {
                self.build_list().await?;
                self.select_first_unread()?;
            }

            Message::Event(ArticlesSelected(augmented_article_filter)) => {
                self.article_filter = Some(augmented_article_filter.clone());
                self.build_list().await?;
                self.select_first_unread()?;
            }

            Message::Command(ArticleListSetScope(scope)) => {
                self.article_scope = *scope;
                self.build_list().await?;
                self.select_first_unread()?;
            }

            Message::Event(ApplicationStateChanged(state)) => {
                self.is_focused = *state == AppState::ArticleSelection;
            }

            Message::Command(ArticleCurrentOpenInBrowser) => {
                self.open_in_browser()?;
            }

            Message::Command(ArticleCurrentSetRead) => {
                self.set_current_read_status(Some(Read::Read)).await?;
                self.build_list().await?;
            }

            Message::Command(ArticleCurrentSetUnread) => {
                self.set_current_read_status(Some(Read::Unread)).await?;
            }

            Message::Command(ArticleCurrentToggleRead) => {
                self.set_current_read_status(None).await?;
            }

            Message::Command(ArticleListSetAllRead) => {
                self.set_all_read_status(Read::Read).await?;
            }

            Message::Command(ArticleListSetAllUnread) => {
                self.set_all_read_status(Read::Unread).await?;
            }

            Message::Event(AsyncMarkArticlesAsReadFinished) => {
                self.build_list().await?;
            }

            Message::Command(ArticleListSelectNextUnread) => {
                // TODO select NEXT unread
                self.select_first_unread()?;
            }

            _ => {}
        }

        let selected_after = self.table_state.selected();

        if selected_before != selected_after
            && let Some(selected) = self.table_state.selected()
            && let Some(article) = self.articles.get(selected)
        {
            self.message_sender
                .send(Message::Event(Event::ArticleSelected(article.clone())))?;
        }

        Ok(())
    }
}
