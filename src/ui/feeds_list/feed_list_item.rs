use crate::prelude::*;
use std::fmt::{Debug, Display};
use std::str::FromStr;

use log::error;
use logos::Logos;
use news_flash::models::{ArticleFilter, Tag};
use news_flash::models::{Category, Feed};
use ratatui::style::Style;
use ratatui::text::{Span, Text};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FeedListItem {
    All,
    Feed(Box<Feed>),
    Categories,
    Category(Box<Category>),
    Tags,
    Tag(Box<Tag>),
    Query(Box<LabeledQuery>),
}

impl Debug for FeedListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FeedListItem as I;
        match self {
            I::All => write!(f, "All"),
            I::Feed(feed) => write!(f, "Feed({})", feed.feed_id),
            I::Categories => write!(f, "Categories"),
            I::Category(category) => write!(f, "Category({})", category.category_id),
            I::Tags => write!(f, "Tags"),
            I::Tag(tag) => write!(f, "Tag({})", tag.label),
            I::Query(query) => write!(f, "Query({})", query.label),
        }
    }
}

#[derive(Clone, Debug, logos::Logos)]
enum LabelToken {
    #[token("{unread_count}", priority = 100)]
    UnreadCount,

    #[token("{marked_count}", priority = 100)]
    MarkedCount,

    #[token("{label}", priority = 100)]
    Label,

    #[regex(r#"[^{]+"#, priority = 0)]
    Fill,
}

impl FeedListItem {
    pub(super) fn to_text<'a>(
        &self,
        config: &Config,
        unread_count: Option<i64>,
        marked_count: Option<i64>,
    ) -> Text<'a> {
        use FeedListItem::*;

        let unread_count_str = unread_count
            .map(|c| if c > 0 { c.to_string() } else { "".to_owned() })
            .unwrap_or_default();

        let marked_count_str = marked_count
            .map(|c| if c > 0 { c.to_string() } else { "".to_owned() })
            .unwrap_or_default();

        let (label, label_template, mut style): (&str, String, Style) = match self {
            All => ("", config.all_label.to_owned(), config.theme.feed()),
            Feed(feed) => (
                feed.label.as_str(),
                config.feed_label.to_owned(),
                config.theme.feed(),
            ),
            Categories => (
                "",
                config.categories_label.to_owned(),
                config.theme.category(),
            ),
            Category(category) => (
                category.label.as_str(),
                config.category_label.to_owned(),
                config.theme.category(),
            ),
            Tags => ("", config.tags_label.to_owned(), config.theme.tag()),
            Tag(tag) => {
                let mut style = config.theme.tag();

                if let Some(color) = NewsFlashUtils::tag_color(tag) {
                    style = style.fg(color);
                }

                (tag.label.as_str(), config.tag_label.to_owned(), style)
            }

            Query(query) => (
                query.label.as_str(),
                config.query_label.to_owned(),
                config.theme.query(),
            ),
        };

        let mut lexer = LabelToken::lexer(label_template.as_str());

        let mut text = Text::default().style(style);

        if let Some(unread_count) = unread_count {
            if unread_count > 0 {
                style = config.theme.patch_unread(&style);
            } else {
                style = config.theme.patch_read(&style);
            }
        }

        while let Some(token) = lexer.next() {
            use LabelToken as T;
            match token {
                Ok(T::Fill) => text.push_span(Span::styled(lexer.slice().to_owned(), style)),
                Ok(T::UnreadCount) => text.push_span(Span::styled(
                    unread_count_str.to_owned(),
                    style.patch(config.theme.unread_count()),
                )),
                Ok(T::MarkedCount) => text.push_span(Span::styled(
                    marked_count_str.to_owned(),
                    style.patch(config.theme.marked_count()),
                )),
                Ok(T::Label) => text.push_span(Span::styled(label.to_owned(), style)),

                Err(_) => error!(
                    "there was an unexpected error while parsing the label template: {label_template}"
                ),
            }
        }

        text
    }

    pub(super) fn to_tooltip(&self, _config: &Config) -> String {
        use FeedListItem::*;
        match self {
            All => "all feeds".to_owned(),
            Categories => "all categories".to_owned(),
            Category(category) => format!("Category: {}", category.label).to_owned(),
            Feed(feed) => {
                format!(
                    "Feed: {} ({})",
                    feed.label,
                    feed.website
                        .as_deref()
                        .map(|url| url.to_string())
                        .unwrap_or("no url".into())
                )
            }
            Tags => "all tagged articles".to_string(),
            Tag(tag) => format!("Tag: {}", tag.label),
            Query(labeled_query) => format!("Query: {}", labeled_query.query),
        }
    }
}

impl TryFrom<FeedListItem> for AugmentedArticleFilter {
    type Error = color_eyre::Report;

    fn try_from(value: FeedListItem) -> Result<Self, Self::Error> {
        use FeedListItem::*;
        Ok(match value {
            All => ArticleFilter::default().into(),
            Feed(feed) => ArticleFilter {
                feeds: vec![feed.feed_id].into(),
                ..Default::default()
            }
            .into(),
            Categories => ArticleFilter::default().into(),
            Category(category) => ArticleFilter {
                categories: vec![category.category_id].into(),
                ..Default::default()
            }
            .into(),
            Tags => AugmentedArticleFilter::from_str("tagged").unwrap(),
            Tag(tag) => ArticleFilter {
                tags: vec![tag.tag_id].into(),
                ..Default::default()
            }
            .into(),
            Query(query) => AugmentedArticleFilter::from_str(&query.query)?,
        })
    }
}

impl Display for FeedListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FeedListItem::*;
        match self {
            All => write!(f, "all"),
            Feed(feed) => write!(f, "feed {}", feed.label),
            Categories => write!(f, "categories"),
            Category(category) => write!(f, "category {}", category.label),
            Tags => write!(f, "tags"),
            Tag(tag) => write!(f, "tag #{}", tag.label),
            Query(labeled_article_query) => {
                write!(f, "article query {}", labeled_article_query.label)
            }
        }
    }
}
