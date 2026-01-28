use getset::Getters;

#[derive(Debug, Clone, serde::Deserialize, Getters)]
#[serde(default)]
pub struct CliConfig {
    pub sync_output_format: String,

    pub all_label_format: String,
    pub feed_label_format: String,
    pub tag_label_format: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            sync_output_format: "{label}:{count}".to_owned(),
            all_label_format: "all:All".to_owned(),
            feed_label_format: "feed:{category}/{label}".to_owned(),
            tag_label_format: "tag:{label}".to_owned(),
        }
    }
}
