use std::collections::HashMap;

use crate::prelude::*;
use itertools::Itertools;
use news_flash::models::FeedID;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SyncStatsOutputFormat {
    pub sync_output_format: String,
    pub all_label_format: String,
    pub feed_label_format: String,
}

impl Default for SyncStatsOutputFormat {
    fn default() -> Self {
        Self::cli_default()
    }
}

impl SyncStatsOutputFormat {
    pub fn cli_default() -> Self {
        Self {
            sync_output_format: "{label}:{count}".to_owned(),
            all_label_format: "all:All".to_owned(),
            feed_label_format: "feed:{category}/{label}".to_owned(),
        }
    }

    pub fn notify_default() -> Self {
        Self {
            sync_output_format: "{count} {label}".to_owned(),
            all_label_format: "New Unread Items".to_owned(),
            feed_label_format: "{category}: {label}".to_owned(),
        }
    }

    pub fn gen_output(
        &self,
        news_flash: &news_flash::NewsFlash,
        new_articles: &HashMap<FeedID, i64>,
    ) -> color_eyre::Result<String> {
        let mut output = String::new();

        let all_unread: i64 = new_articles.values().sum();

        let (
            mut feeds,
            feed_for_feed_id,
            feed_mapping_for_feed_id,
            mut categories,
            category_for_category_id,
            category_mapping_for_category_id,
        ) = get_feeds_and_categories(news_flash)?;

        sort_feeds_and_categories(
            &mut feeds,
            &mut categories,
            &feed_mapping_for_feed_id,
            &category_mapping_for_category_id,
        );

        if all_unread > 0 {
            output.push_str(
                &self
                    .sync_output_format
                    .replace("{label}", &self.all_label_format)
                    .replace("{count}", &all_unread.to_string()),
            );
            output.push('\n');
        }

        if !self.feed_label_format.is_empty() {
            output.push_str(
                &feeds
                    .iter()
                    .filter(|feed| *new_articles.get(&feed.feed_id).unwrap_or(&0) > 0)
                    .map(|feed| {
                        self.sync_output_format
                            .replace("{label}", &self.feed_label_format)
                            .replace(
                                "{category}",
                                feed_mapping_for_feed_id
                                    .get(&feed.feed_id)
                                    .and_then(|mapping| {
                                        category_for_category_id
                                            .get(&mapping.category_id)
                                            .map(|category| &*category.label)
                                    })
                                    .unwrap_or_default(),
                            )
                            .replace(
                                "{label}",
                                feed_for_feed_id
                                    .get(&feed.feed_id)
                                    .map(|feed| &feed.label)
                                    .unwrap_or(&"".to_owned()),
                            )
                            .replace(
                                "{count}",
                                &new_articles.get(&feed.feed_id).unwrap_or(&0).to_string(),
                            )
                    })
                    .join("\n"),
            );
        }

        Ok(output)
    }
}
