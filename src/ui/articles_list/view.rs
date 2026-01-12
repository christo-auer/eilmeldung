use crate::prelude::*;
use crate::ui::articles_list::model::ArticleListModelData;
use std::sync::Arc;

use getset::{Getters, MutGetters};
use news_flash::models::{ArticleFilter, Marked, Read};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table,
    TableState, Widget,
};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
};
use strum::IntoEnumIterator;

#[derive(Getters, MutGetters)]
#[getset(get = "pub(super)")]
pub struct FilterState {
    default_sort_order: SortOrder,

    augmented_article_filter: Option<AugmentedArticleFilter>,

    #[get_mut = "pub(super)"]
    article_scope: ArticleScope,

    #[get_mut = "pub(super)"]
    article_search_query: Option<ArticleQuery>,

    #[get_mut = "pub(super)"]
    article_adhoc_filter: Option<ArticleQuery>,

    #[get_mut = "pub(super)"]
    adhoc_sort_order: Option<SortOrder>,

    #[get_mut = "pub(super)"]
    reverse_sort_order: bool,

    #[get_mut = "pub(super)"]
    apply_article_adhoc_filter: bool,
}

// impl Default for FilterState {
//     fn default() -> Self {
//         Self {
//             default_sort_order: SortOrder::default(),
//             article_scope: ArticleScope::All,
//             augmented_article_filter: None,
//             article_search_query: None,
//             article_adhoc_filter: None,
//             apply_article_adhoc_filter: false,
//             adhoc_sort_order: None,
//             reverse_sort_order: false,
//         }
//     }
// }

impl FilterState {
    pub fn new(article_scope: ArticleScope, default_sort_order: SortOrder) -> Self {
        Self {
            default_sort_order,
            article_scope,
            augmented_article_filter: None,
            article_search_query: None,
            article_adhoc_filter: None,
            adhoc_sort_order: None,
            apply_article_adhoc_filter: false,
            reverse_sort_order: false,
        }
    }

    pub(super) fn generate_effective_filter(&self) -> Option<ArticleFilter> {
        let augmented_article_filter = self.augmented_article_filter.as_ref()?;

        let mut article_filter = augmented_article_filter.article_filter.clone();

        // read/unread/marked etc comes from query
        if !augmented_article_filter.defines_scope() {
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

    pub fn get_effective_scope(&self) -> Option<ArticleScope> {
        if let Some(augmented_article_filter) = self.augmented_article_filter.as_ref()
            && augmented_article_filter.defines_scope()
        {
            return None;
        }
        Some(self.article_scope)
    }

    pub fn uses_default_sort_order(&self) -> bool {
        self.adhoc_sort_order.is_none()
            && (self
                .augmented_article_filter
                .as_ref()
                .is_none_or(|filter| filter.article_query.sort_order().is_none()))
            && !*self.reverse_sort_order()
    }

    pub fn get_effective_sort_order(&self) -> SortOrder {
        self.adhoc_sort_order
            .as_ref()
            .or_else(|| {
                self.article_adhoc_filter
                    .as_ref()
                    .and_then(|filter| filter.sort_order().as_ref())
            })
            .or_else(|| {
                self.augmented_article_filter
                    .as_ref()
                    .and_then(|filter| filter.article_query.sort_order().as_ref())
            })
            .unwrap_or(&self.default_sort_order)
            .to_owned()
            .reverse(self.reverse_sort_order)
    }

    pub fn on_new_article_filter(&mut self, article_filter: AugmentedArticleFilter) {
        self.augmented_article_filter = Some(article_filter);
        self.apply_article_adhoc_filter = false;
    }

    pub fn on_new_article_adhoc_filter(&mut self, article_adhoc_filter: ArticleQuery) {
        self.article_adhoc_filter = Some(article_adhoc_filter);
        self.apply_article_adhoc_filter = true;
    }

    pub fn clear_sort_order(&mut self) {
        self.adhoc_sort_order = None;
        self.reverse_sort_order = false;
    }
}

impl Widget for &mut ArticlesList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = self
            .view_data
            .gen_block(&self.config, &self.filter_state, self.is_focused);
        let inner = block.inner(area);

        block.render(area, buf);

        *self.view_data.article_lines_mut() = Some(area.height.saturating_sub(1));

        StatefulWidget::render(
            &self.view_data.table,
            inner,
            buf,
            &mut self.view_data.table_state,
        );

        // let scroll_thumb_icon = self.config.scroll_thumb_icon.to_string();
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(self.config.scrollbar_set())
            .style(self.config.theme.eff_border(self.is_focused));

        let scrollbar_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(1),
        };
        StatefulWidget::render(
            scrollbar,
            scrollbar_area,
            buf,
            &mut self.view_data.scrollbar_state,
        );
    }
}

#[derive(Default, Getters, MutGetters)]
#[getset(get = "pub(super)")]
pub struct ArticleListViewData<'a> {
    table: Table<'a>,
    #[getset(get_mut = "pub(super)")]
    table_state: TableState,

    #[getset(get_mut = "pub(super)")]
    scrollbar_state: ScrollbarState,

    #[getset(get_mut = "pub(super)", get = "pub(super)")]
    article_lines: Option<u16>,
}

impl<'a> ArticleListViewData<'a> {
    fn build_title(&self, filter_state: &FilterState, config: &Config) -> Line<'static> {
        let mut title = Line::styled("", config.theme.header());
        let spans = &mut title.spans;

        if let Some(article_scope) = filter_state.get_effective_scope() {
            for scope in ArticleScope::iter() {
                let style = if scope == article_scope {
                    config.theme.header()
                } else {
                    config.theme.inactive()
                };
                spans.push(" ".into());
                spans.push(Span::styled(scope.to_icon(config).to_string(), style));
            }
            spans.push(" ".into());
        }

        let filter_info = match filter_state.article_adhoc_filter {
            Some(_) if filter_state.apply_article_adhoc_filter => "  ",
            Some(_) => "  ",
            _ => "",
        };

        spans.push(Span::styled(filter_info, config.theme.header()));

        if !filter_state.uses_default_sort_order() {
            let filter_text = &format!(
                " {} {} ",
                if *filter_state.reverse_sort_order() {
                    "󰒿"
                } else {
                    "󰌼"
                },
                filter_state.get_effective_sort_order()
            );
            spans.push(Span::styled(filter_text.to_owned(), config.theme.header()));
        }
        // spans.push(Span::styled("├", config.theme.header()));

        title
    }

    pub fn update(
        &mut self,
        config: Arc<Config>,
        model_data: &ArticleListModelData,
        filter_state: &FilterState,
        _is_focused: bool,
    ) {
        let selected_style = Style::new().reversed();

        let read_icon = config.read_icon.to_string();
        let unread_icon = config.unread_icon.to_string();
        let marked_icon = config.marked_icon.to_string();
        let unmarked_icon = config.unmarked_icon.to_string();

        let placeholders: Vec<&str> = config
            .article_table
            .split(",")
            .map(|placeholder| placeholder.trim())
            .collect();

        let mut max_tags: u16 = 0;

        let entries: Vec<Row> = model_data
            .articles()
            .iter()
            .map(|article| {
                let row_vec: Vec<Line> = placeholders
                    .iter()
                    .map(|placeholder| match *placeholder {
                        "{title}" => {
                            html_sanitize(article.title.as_deref().unwrap_or("no title")).into()
                        }
                        "{tag_icons}" => Line::from(
                            match model_data.tags_for_article().get(&article.article_id) {
                                Some(tag_ids) => {
                                    max_tags = u16::max(max_tags, tag_ids.len() as u16);

                                    tag_ids
                                        .iter()
                                        .map(|tag_id| {
                                            let Some(tag) = model_data.tag_map().get(tag_id) else {
                                                return Span::from("");
                                            };

                                            let style = match NewsFlashUtils::tag_color(tag) {
                                                Some(color) => config.theme.tag().fg(color),
                                                None => config.theme.tag(),
                                            };
                                            Span::styled(config.tag_icon.to_string(), style)
                                        })
                                        .collect::<Vec<Span>>()
                                }
                                None => vec![Span::from("")],
                            },
                        ),
                        "{author}" => {
                            html_sanitize(article.author.as_deref().unwrap_or("no author")).into()
                        }
                        "{feed}" => html_sanitize(
                            model_data
                                .feed_map()
                                .get(&article.feed_id)
                                .map(|feed| feed.label.as_str())
                                .unwrap_or("unknown feed"),
                        )
                        .into(),
                        "{date}" => article
                            .date
                            .with_timezone(&chrono::Local)
                            .format(&config.date_format)
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

                let style = match filter_state.article_search_query.as_ref() {
                    Some(query)
                        if query.test(
                            article,
                            model_data.feed_map(),
                            model_data.tags_for_article(),
                            model_data.tag_map(),
                        ) =>
                    {
                        config.theme.article_highlighted()
                    }
                    _ => config.theme.article(),
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
                Constraint::Length(config.date_format.len() as u16)
            } else if placeholder == "{tag_icons}" {
                Constraint::Length(max_tags)
            } else {
                Constraint::Min(1)
            }
        };

        self.scrollbar_state = self
            .scrollbar_state
            .content_length(entries.len())
            .position(0);

        self.table = Table::new(
            entries,
            placeholders
                .iter()
                .map(|placeholder| constraint_for_placeholder(placeholder))
                .collect::<Vec<Constraint>>(),
        )
        .row_highlight_style(selected_style);
    }

    pub(super) fn gen_block(
        &self,
        config: &Config,
        filter_state: &FilterState,
        is_focused: bool,
    ) -> Block<'static> {
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title_top(self.build_title(filter_state, config))
            .title_alignment(ratatui::layout::Alignment::Left)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(if is_focused {
                config.theme.border_focused()
            } else {
                config.theme.border()
            })
    }

    pub(super) fn get_table_state_mut(&mut self) -> &mut TableState {
        &mut self.table_state
    }

    pub(super) fn get_table_state(&self) -> &TableState {
        &self.table_state
    }
}
