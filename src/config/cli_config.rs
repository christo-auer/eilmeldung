use getset::Getters;

#[derive(Debug, Clone, serde::Deserialize, Getters)]
#[serde(default)]
pub struct CliConfig {
    pub sync_output_format: String,
    pub stats_output_format: String,
    pub stats_format: String,

    pub all_label_format: String,
    pub feed_label_format: String,
    pub category_label_format: String,
    pub tag_label_format: String,

    pub sync_output_all: bool,
    pub sync_output_feeds: bool,

}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            sync_output_format: "{label}:{count}".to_owned(),
            stats_output_format: "{label}:{stats}".to_owned(),
            stats_format: "{all}:{unread}".to_owned(),
            all_label_format: "all:All".to_owned(),
            feed_label_format: "feed:{category}/{label}".to_owned(),
            category_label_format: "category:{label}".to_owned(),
            tag_label_format: "tag:{label}".to_owned(),
            sync_output_all: true,
            sync_output_feeds: true,
        }
    }
}
