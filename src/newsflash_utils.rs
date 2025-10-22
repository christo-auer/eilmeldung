use std::{collections::HashMap, hash::Hash, str::FromStr, sync::Arc};

use news_flash::{
    NewsFlash,
    models::{ArticleID, Category, CategoryID, Feed, FeedID, Read, Tag, TagID, Tagging},
};

use log::{debug, error, info};
use ratatui::style::{Color, Style};
use ratatui_image::FilterType::Triangle;
use reqwest::Client;
use tokio::sync::{Mutex, RwLock, mpsc::UnboundedSender};

use crate::commands::{Event, Message};

#[derive(Clone)]
pub struct NewsFlashUtils {
    pub news_flash_lock: Arc<RwLock<NewsFlash>>,
    client_lock: Arc<RwLock<Client>>,
    command_sender: UnboundedSender<Message>,

    async_operation_mutex: Arc<Mutex<()>>,
}

impl NewsFlashUtils {
    pub fn new(
        news_flash: NewsFlash,
        client: Client,
        command_sender: UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating NewsFlashUtils");
        Self {
            news_flash_lock: Arc::new(RwLock::new(news_flash)),
            client_lock: Arc::new(RwLock::new(client)),
            command_sender,
            async_operation_mutex: Arc::new(Mutex::new(())),
        }
    }

    // for polling
    pub fn is_async_operation_running(&self) -> bool {
        self.async_operation_mutex.try_lock().is_err()
    }

    pub fn sync_feeds(&self) {
        info!("Starting feed sync operation");
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            debug!("Acquiring async operation lock for sync");
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                debug!("Sending AsyncSyncStarted command");
                command_sender.send(Message::Event(Event::AsyncSyncStarted))?;

                debug!("Acquiring NewsFlash and client locks");
                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                info!("Starting NewsFlash sync operation");
                let new_articles = news_flash.sync(&client, Default::default()).await?;
                info!("Sync completed, {} new articles found", new_articles.len());

                debug!("Sending AsyncSyncFinished command");
                command_sender.send(Message::Event(Event::AsyncSyncFinished(new_articles)))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!("Feed sync failed: {}", e);
                let _ =
                    command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
            }
        });
    }

    pub fn fetch_thumbnail(&self, article_id: ArticleID) {
        debug!("Starting thumbnail fetch for article: {:?}", article_id);
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            debug!("Acquiring async operation lock for thumbnail fetch");
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                debug!("Sending AsyncFetchThumbnailStarted command");
                command_sender.send(Message::Event(Event::AsyncFetchThumbnailStarted))?;

                debug!("Acquiring NewsFlash and client locks for thumbnail");
                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                debug!("Fetching thumbnail from NewsFlash");
                let thumbnail = news_flash
                    .get_article_thumbnail(&article_id, &client)
                    .await?;

                match &thumbnail {
                    Some(_) => info!(
                        "Thumbnail fetched successfully for article: {:?}",
                        article_id
                    ),
                    None => debug!("No thumbnail available for article: {:?}", article_id),
                }

                debug!("Sending AsyncFetchThumbnailFinished command");
                command_sender.send(Message::Event(Event::AsyncFetchThumbnailFinished(
                    thumbnail,
                )))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!("Thumbnail fetch failed for article {:?}: {}", article_id, e);
                let _ =
                    command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
            }
        });
    }

    pub fn fetch_fat_article(&self, article_id: ArticleID) {
        info!("Starting fat article fetch for article: {:?}", article_id);
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            debug!("Acquiring async operation lock for fat article fetch");
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                debug!("Sending AsyncFetchFatArticleStarted command");
                command_sender.send(Message::Event(Event::AsyncFetchFatArticleStarted))?;

                debug!("Acquiring NewsFlash and client locks for fat article");
                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                info!("Scraping article content from NewsFlash");
                let fat_article = news_flash
                    .scrap_content_article(&article_id, &client, None)
                    .await?;

                info!(
                    "Fat article fetched successfully for article: {:?}",
                    article_id
                );
                debug!("Sending AsyncFetchFatArticleFinished command");
                command_sender.send(Message::Event(Event::AsyncFetchFatArticleFinished(
                    fat_article,
                )))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!(
                    "Fat article fetch failed for article {:?}: {}",
                    article_id, e
                );
                let _ =
                    command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
            }
        });
    }

    pub fn set_article_status(&self, article_ids: Vec<ArticleID>, read: Read) {
        info!(
            "Starting article status update: {} articles to {:?}",
            article_ids.len(),
            read
        );
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            debug!("Acquiring async operation lock for article status update");
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                debug!("Sending AsyncMarkArticlesAsReadStarted command");
                command_sender.send(Message::Event(Event::AsyncMarkArticlesAsReadStarted))?;

                debug!("Acquiring NewsFlash and client locks for article status");
                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                debug!("Updating article read status in NewsFlash");
                news_flash
                    .set_article_read(&article_ids, read, &client)
                    .await?;

                info!(
                    "Successfully updated status for {} articles",
                    article_ids.len()
                );
                debug!("Sending AsyncMarkArticlesAsReadFinished command");
                command_sender.send(Message::Event(Event::AsyncMarkArticlesAsReadFinished))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!(
                    "Article status update failed for {} articles: {}",
                    article_ids.len(),
                    e
                );
                let _ =
                    command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
            }
         });
    }

    /// Get icon for a tag based on its label
    pub fn get_tag_icon(tag_label: &str) -> &'static str {
        match tag_label.to_lowercase().as_str() {
            "important" | "urgent" | "priority" => "🔥",
            "tech" | "technology" | "programming" => "💻",
            "news" | "breaking" => "📰",
            "science" | "research" => "🔬",
            "business" | "finance" => "💼",
            "sports" => "⚽",
            "entertainment" | "media" => "🎬",
            "health" | "medical" => "🏥",
            "travel" => "✈️",
            "food" | "cooking" => "🍽️",
            "politics" => "🏛️",
            "environment" | "climate" => "🌍",
            "education" | "learning" => "📚",
            "art" | "design" => "🎨",
            "music" => "🎵",
            "gaming" | "games" => "🎮",
            "crypto" | "blockchain" | "bitcoin" => "₿",
            "ai" | "ml" | "artificial intelligence" | "machine learning" => "🤖",
            "security" | "cybersecurity" => "🔒",
            "open source" | "oss" => "🔓",
            _ => "🏷️",
        }
    }

    /// Get color for a tag based on its label  
    pub fn get_tag_color(tag_label: &str) -> &'static str {
        match tag_label.to_lowercase().as_str() {
            "important" | "urgent" | "priority" => "#ff4444", // red
            "tech" | "technology" | "programming" => "#0066cc", // blue
            "news" | "breaking" => "#ff6600", // orange
            "science" | "research" => "#9933cc", // purple
            "business" | "finance" => "#006600", // green
            "sports" => "#ffaa00", // amber
            "entertainment" | "media" => "#cc3399", // magenta
            "health" | "medical" => "#00aa55", // teal
            "travel" => "#3366ff", // light blue
            "food" | "cooking" => "#ff9900", // orange
            "politics" => "#666666", // gray
            "environment" | "climate" => "#228833", // forest green
            "education" | "learning" => "#4455aa", // indigo
            "art" | "design" => "#aa4488", // pink
            "music" => "#8844aa", // violet
            "gaming" | "games" => "#aa2222", // dark red
            "crypto" | "blockchain" | "bitcoin" => "#f7931a", // bitcoin orange
            "ai" | "ml" | "artificial intelligence" | "machine learning" => "#00cccc", // cyan
            "security" | "cybersecurity" => "#aa0000", // dark red
            "open source" | "oss" => "#228b22", // forest green
            _ => "#888888", // default gray
        }
    }

    pub fn generate_id_map<V, I: Hash + Eq + Clone>(
        items: &Vec<V>,
        id_extractor: impl Fn(&V) -> I,
    ) -> HashMap<I, V>
    where
        V: Clone,
    {
        items
            .iter()
            .map(|item| (id_extractor(item), item.clone()))
            .collect()
    }

    pub fn generate_one_to_many<E, I: Hash + Eq + Clone, V>(
        mappings: &Vec<E>,
        id_extractor: impl Fn(&E) -> I,
        value_extractor: impl Fn(&E) -> V,
    ) -> HashMap<I, Vec<V>>
    where
        V: Clone,
    {
        mappings
            .into_iter()
            .fold(HashMap::new(), |mut acc, mapping| {
                acc.entry(id_extractor(mapping).clone())
                    .or_default()
                    .push(value_extractor(mapping).clone());
                acc
            })
    }

    pub fn tag_color(tag: &Tag) -> Option<Color> {
        if let Some(color_str) = tag.color.clone()
            && let Ok(tag_color) = Color::from_str(color_str.as_str())
        {
            return Some(tag_color);
        }

        None
    }
}
