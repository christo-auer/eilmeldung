use news_flash::models::{Article, ArticleID, Enclosure, FatArticle, Feed, Tag, Thumbnail};

#[derive(getset::Getters, getset::MutGetters)]
#[getset(get = "pub(super)", get_mut = "pub(super)")]
pub(super) struct LoadedArticle {
    pub(super) article: Article,
    pub(super) feed: Feed,
    pub(super) tags: Vec<Tag>,
    pub(super) enclosures: Vec<Enclosure>,
    pub(super) thumbnail_state: ThumbnailState,
    pub(super) body: ArticleBody,
}

#[derive(enum_as_inner::EnumAsInner, Default, Debug)]
pub(super) enum ArticleBody {
    #[default]
    Summary,
    Fat {
        fat_article: Box<FatArticle>,
        markdown_content: String,
    },
    Piped {
        piped_content: String,
    },
}

#[derive(enum_as_inner::EnumAsInner)]
pub enum ThumbnailState {
    None,
    Fetching,
    Fetched(Box<Thumbnail>),
    Failed,
}

impl LoadedArticle {
    pub(super) fn article_id(&self) -> &ArticleID {
        &self.article.article_id
    }

    pub(super) fn is(&self, article_id: &ArticleID) -> bool {
        self.article_id() == article_id
    }
    pub(super) fn content(&self) -> &str {
        use ArticleBody as B;
        match self.body {
            B::Summary => self.article.summary.as_deref().unwrap_or(""),
            B::Fat {
                ref markdown_content,
                ..
            } => &*markdown_content,
            B::Piped { ref piped_content } => &*piped_content,
        }
    }
}
