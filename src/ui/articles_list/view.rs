use crate::prelude::*;
use std::sync::Arc;

use super::message_recv::ArticleListModelData;

use news_flash::models::{Article, ArticleFilter, ArticleID, Marked, Read};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
};

pub struct FilterState {
    pub(super) augmented_article_filter: Option<AugmentedArticleFilter>,
    pub(super) article_scope: ArticleScope,
    pub(super) article_search_query: Option<ArticleQuery>,
    pub(super) article_adhoc_filter: Option<ArticleQuery>,
    pub(super) apply_article_adhoc_filter: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            article_scope: ArticleScope::All,
            augmented_article_filter: None,
            article_search_query: None,
            article_adhoc_filter: None,
            apply_article_adhoc_filter: false,
        }
    }
}

impl FilterState {
    pub fn new(article_scope: ArticleScope) -> Self {
        Self {
            article_scope,
            augmented_article_filter: None,
            article_search_query: None,
            article_adhoc_filter: None,
            apply_article_adhoc_filter: false,
        }
    }

    pub fn get_effective_scope(&self) -> Option<ArticleScope> {
        if let Some(augmented_article_filter) = self.augmented_article_filter.as_ref()
            && augmented_article_filter.is_augmented()
        {
            return None;
        }
        Some(self.article_scope)
    }

    pub fn on_new_article_filter(&mut self, article_filter: AugmentedArticleFilter) {
        self.augmented_article_filter = Some(article_filter);
        self.apply_article_adhoc_filter = false;
    }

    pub fn on_new_article_adhoc_filter(&mut self, article_adhoc_filter: ArticleQuery) {
        self.article_adhoc_filter = Some(article_adhoc_filter);
        self.apply_article_adhoc_filter = true;
    }

    pub async fn fill_articles(
        &self,
        news_flash_utils: Arc<NewsFlashUtils>,
        model_data: &mut ArticleListModelData,
    ) -> color_eyre::Result<()> {
        let Some(augmented_article_filter) = self.augmented_article_filter.as_ref() else {
            return Ok(());
        };

        let Some(mut article_filter) = self.generate_effective_filter() else {
            return Ok(());
        };

        let news_flash = news_flash_utils.news_flash_lock.read().await;

        // TODO make configurable
        article_filter.order_by = Some(news_flash::models::OrderBy::Published);
        article_filter.order = Some(news_flash::models::ArticleOrder::NewestFirst);

        model_data.articles = news_flash.get_articles(article_filter.clone())?;

        if augmented_article_filter.is_augmented() {
            model_data.articles = augmented_article_filter.article_query.filter(
                &model_data.articles,
                &model_data.feed_map,
                &model_data.tags_for_article,
                &model_data.tag_map,
            );
        }

        if let Some(article_adhoc_filter) = self.article_adhoc_filter.as_ref()
            && self.apply_article_adhoc_filter
        {
            model_data.articles = article_adhoc_filter.filter(
                &model_data.articles,
                &model_data.feed_map,
                &model_data.tags_for_article,
                &model_data.tag_map,
            );
        }

        Ok(())
    }

    fn generate_effective_filter(&self) -> Option<ArticleFilter> {
        let augmented_article_filter = self.augmented_article_filter.as_ref()?;

        let mut article_filter = augmented_article_filter.article_filter.clone();

        // read/unread/marked etc comes from query
        if !augmented_article_filter.is_augmented() {
            match self.article_scope {
                ArticleScope::All => {}
                ArticleScope::Unread => {
                    article_filter.unread = Some(Read::Unread);
                    article_filter.marked = None;
                }
                ArticleScope::Marked => {
                    article_filter.marked = Some(Marked::Marked);
                    article_filter.unread = None;
                }
            }
        }
        Some(article_filter)
    }
}

#[derive(Default)]
pub struct ArticleListViewData {
    pub(super) table: Table<'static>,
    pub(super) table_state: TableState,
    pub(super) is_focused: bool,
    pub(super) filter_state: FilterState,
}

impl ArticleListViewData {
    pub(super) fn new(article_scope: ArticleScope) -> Self {
        Self {
            filter_state: FilterState::new(article_scope),
            ..Self::default()
        }
    }
}

impl ArticlesList {
    pub(super) fn restore_sensible_selection(
        &mut self,
        article_id: Option<&ArticleID>,
    ) -> color_eyre::Result<()> {
        // save offset distance
        let offset = *self.view_data.table_state.offset_mut();
        let offset_distance = self
            .view_data
            .table_state
            .selected()
            .unwrap_or(0)
            .saturating_sub(offset);

        // first, we try to select the article with article_id
        if let Some(article_id) = article_id
            && let Some(index) = self
                .model_data
                .articles
                .iter()
                .position(|article| article.article_id == *article_id)
        {
            *self.view_data.table_state.offset_mut() = index.saturating_sub(offset_distance);
            return self.select_index_and_send_message(Some(index));
        }

        // the previous article is not there, next we select the first unread article
        self.view_data.table_state.select(Some(0));
        self.select_next_unread()?;

        Ok(())
    }

    pub(super) fn select_index_and_send_message(
        &mut self,
        index: Option<usize>,
    ) -> color_eyre::Result<()> {
        let index = index
            .or(self.view_data.table_state.selected())
            .unwrap_or_default();
        if let Some(article) = self.model_data.articles.get(index) {
            self.view_data.table_state.select(Some(index));
            self.message_sender
                .send(Message::Event(Event::ArticleSelected(
                    article.clone(),
                    self.model_data.feed_map.get(&article.feed_id).cloned(),
                    self.model_data
                        .tags_for_article
                        .get(&article.article_id)
                        .map(|tag_ids| {
                            tag_ids
                                .iter()
                                .filter_map(|tag_id| self.model_data.tag_map.get(tag_id))
                                .cloned()
                                .collect()
                        })
                        .clone(),
                )))?;
        }

        self.adjust_offset();
        Ok(())
    }

    pub(super) fn select_next_unread(&mut self) -> color_eyre::Result<()> {
        let select = self.first_unread();
        self.select_index_and_send_message(select)
    }

    fn first_unread(&self) -> Option<usize> {
        let current_index = self.view_data.table_state.selected().unwrap_or(0);

        self.model_data
            .articles
            .iter()
            .enumerate()
            .find(|(index, article)| *index >= current_index && article.unread == Read::Unread)
            .map(|(index, _)| index)
    }

    pub(super) fn open_in_browser(&self) -> color_eyre::Result<()> {
        if let Some(article) = self.get_current_article()
            && let Some(url) = article.url.as_ref()
        {
            webbrowser::open(url.as_ref())?;
        }

        // TODO error handling
        Ok(())
    }

    pub(super) fn get_current_article_mut(&mut self) -> Option<&mut Article> {
        if let Some(index) = self.view_data.table_state.selected() {
            return self.model_data.articles.get_mut(index);
        }

        None
    }

    pub(super) fn get_current_article(&self) -> Option<Article> {
        if let Some(index) = self.view_data.table_state.selected() {
            return self.model_data.articles.get(index).cloned();
        }

        None
    }

    fn build_title(&self) -> String {
        let mut title = String::new();
        let filter_state = &self.view_data.filter_state;

        if let Some(article_scope) = filter_state.get_effective_scope() {
            let to_icon = |scope: ArticleScope| -> char {
                if scope == article_scope {
                    '󰐾'
                } else {
                    ''
                }
            };

            title.push_str(&format!(
                " {} All {} Unread {} Marked ",
                to_icon(ArticleScope::All),
                to_icon(ArticleScope::Unread),
                to_icon(ArticleScope::Marked)
            ));
        }

        let filter_info = match filter_state.article_adhoc_filter {
            Some(_) if filter_state.apply_article_adhoc_filter => " ",
            Some(_) => " ",
            _ => "",
        };

        title.push_str(filter_info);
        title
    }

    fn adjust_offset(&mut self) {
        let Some(index) = self.view_data.table_state.selected() else {
            return;
        };
        let offset = self.view_data.table_state.offset_mut();
        let max_lines_above = self.config.theme.articles_list_height_lines as usize
            - (self.config.articles_list_visible_articles_after_selection + 1);

        if index.saturating_sub(*offset) > max_lines_above {
            *offset = index.saturating_sub(max_lines_above);
        }
    }

    pub(super) fn search(
        &self,
        articles: &[Article],
        article_query: &ArticleQuery,
        reversed: bool,
    ) -> Option<usize> {
        let predicate = |article: &Article| {
            article_query.test(
                article,
                &self.model_data.feed_map,
                &self.model_data.tags_for_article,
                &self.model_data.tag_map,
            )
        };

        if !reversed {
            articles.iter().position(predicate)
        } else {
            articles.iter().rposition(predicate)
        }
    }

    pub(super) fn search_next(
        &mut self,
        skip_current: bool,
        reversed: bool,
    ) -> color_eyre::Result<()> {
        let offset = if skip_current { 1 } else { 0 };
        let Some(article_query) = self.view_data.filter_state.article_search_query.as_ref() else {
            return Err(color_eyre::eyre::eyre!("no search query"));
        };

        if let Some(selected) = self.view_data.table_state.selected() {
            let split_index = if !reversed {
                selected + offset
            } else {
                selected.saturating_sub(offset)
            };

            let slices = self.model_data.articles.split_at(split_index);

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
                    return self.select_index_and_send_message(Some(index + first_offset));
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
                        return self.select_index_and_send_message(Some(index + second_offset));
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

    pub(super) fn update_view_data(&mut self) {
        let selected_style = if self.view_data.is_focused {
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
            .model_data
            .articles
            .iter()
            .map(|article| {
                let row_vec: Vec<Line> = placeholders
                    .iter()
                    .map(|placeholder| match *placeholder {
                        "{title}" => article.title.as_deref().unwrap_or("?").to_string().into(),
                        "{tag_icons}" => Line::from(
                            match self.model_data.tags_for_article.get(&article.article_id) {
                                Some(tag_ids) => {
                                    max_tags = u16::max(max_tags, tag_ids.len() as u16);

                                    tag_ids
                                        .iter()
                                        .map(|tag_id| {
                                            let Some(tag) = self.model_data.tag_map.get(tag_id)
                                            else {
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
                            },
                        ),
                        "{author}" => article.author.as_deref().unwrap_or("?").to_string().into(),
                        "{feed}" => self
                            .model_data
                            .feed_map
                            .get(&article.feed_id)
                            .map(|feed| feed.label.as_str())
                            .unwrap_or("?")
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
                            .as_ref()
                            .map(|url| url.to_string())
                            .unwrap_or("?".into())
                            .into(),
                        _ => format!("{placeholder}?").into(),
                    })
                    .collect();

                let style = match self.view_data.filter_state.article_search_query.as_ref() {
                    Some(query)
                        if query.test(
                            article,
                            &self.model_data.feed_map,
                            &self.model_data.tags_for_article,
                            &self.model_data.tag_map,
                        ) =>
                    {
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

        self.view_data.table = Table::new(
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
                .title_top(self.build_title())
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(if self.view_data.is_focused {
                    self.config.theme.focused_border_style
                } else {
                    self.config.theme.border_style
                }),
        );
    }
}

impl Widget for &mut ArticlesList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        StatefulWidget::render(
            &self.view_data.table,
            area,
            buf,
            &mut self.view_data.table_state,
        );
    }
}
