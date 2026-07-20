use std::fmt::Display;

use news_flash::models::{ArticleID, Marked, Read, TagID};

pub mod prelude {
    pub use super::UndoOperation;
}

#[derive(Clone)]
pub enum UndoOperation {
    ChangeRead(Vec<ArticleID>, Read),
    ChangeMarked(Vec<ArticleID>, Marked),
    AddTag(Vec<ArticleID>, TagID),
    RemoveTag(Vec<ArticleID>, TagID),
}

impl Display for UndoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UndoOperation::ChangeRead(articles, read) => {
                let read_str = if matches!(read, Read::Read) {
                    "read"
                } else {
                    "unread"
                };
                write!(f, "undo set {} articles as {read_str}", articles.len())
            }
            UndoOperation::ChangeMarked(articles, marked) => {
                let marked_str = if matches!(marked, Marked::Marked) {
                    "marked"
                } else {
                    "unmarked"
                };
                write!(f, "undo set {} articles as {marked_str}", articles.len())
            }
            UndoOperation::AddTag(article_ids, _) => {
                write!(f, "undo add tag to {}", article_ids.len())
            }
            UndoOperation::RemoveTag(article_ids, _) => {
                write!(f, "undo remove tag from {} articles", article_ids.len())
            }
        }
    }
}
