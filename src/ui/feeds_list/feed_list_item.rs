use crate::prelude::*;
use std::str::FromStr;

use news_flash::models::{ArticleFilter, Tag, TagID};
use news_flash::models::{Category, Feed};
use ratatui::text::Text;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FeedListItem {
    All,
    Feed(Box<Feed>),
    Category(Box<Category>),
    Tags(Vec<TagID>),
    Tag(Box<Tag>),
    Query(Box<LabeledQuery>),
}

impl FeedListItem {
    pub(super) fn to_text<'a>(
        &self,
        config: &Config,
        unread_count: Option<i64>,
        marked_count: Option<i64>,
    ) -> Text<'a> {
        use FeedListItem::*;

        let unread_count_str = unread_count.map(|c| c.to_string()).unwrap_or_default();

        let marked_count_str = marked_count.map(|c| c.to_string()).unwrap_or_default();

        let (label, mut style) = match self {
            All => (config.all_label.to_string(), config.theme.header),
            Feed(feed) => (
                config.feed_label.replace("{label}", feed.label.as_str()),
                config.theme.feed,
            ),
            Category(category) => (
                config
                    .category_label
                    .replace("{label}", category.label.as_str()),
                config.theme.category,
            ),
            Tags(_) => (config.tags_label.to_string(), config.theme.header),
            Tag(tag) => {
                let mut style = config.theme.tag;

                let color = NewsFlashUtils::tag_color(tag).unwrap_or(style.fg.unwrap());
                style = style.fg(color);

                let label = config.tag_label.replace("{label}", &tag.label);

                (label, style)
            }

            Query(query) => (
                config.query_label.replace("{label}", &query.label),
                config.theme.query,
            ),
        };

        if let Some(unread_count) = unread_count
            && unread_count > 0
        {
            style = style.add_modifier(config.theme.unread_modifier);
        }

        Text::styled(
            label
                .replace("{unread_count}", unread_count_str.as_str())
                .replace("{marked_count}", marked_count_str.as_str()),
            style,
        )
    }

    pub(super) fn to_tooltip(&self, _config: &Config) -> String {
        use FeedListItem::*;
        match self {
            All => "all feeds".to_string(),
            Category(category) => format!("Category: {}", category.label).to_string(),
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
            Tags(_) => "all tagged articles".to_string(),
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
            Category(category) => ArticleFilter {
                categories: vec![category.category_id].into(),
                ..Default::default()
            }
            .into(),
            Tags(tag_ids) => ArticleFilter {
                tags: Some(tag_ids),
                ..Default::default()
            }
            .into(),
            Tag(tag) => ArticleFilter {
                tags: vec![tag.tag_id].into(),
                ..Default::default()
            }
            .into(),
            Query(query) => AugmentedArticleFilter::from_str(&query.query)?,
        })
    }
}
