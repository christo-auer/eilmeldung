use std::sync::Arc;

use news_flash::{
    NewsFlash,
    models::{ArticleID, Read},
};

use log::{debug, error, info};
use reqwest::Client;
use tokio::sync::{Mutex, RwLock, mpsc::UnboundedSender};

use crate::commands::{Message, Event};

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
                let _ = command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
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
                command_sender.send(Message::Event(Event::AsyncFetchThumbnailFinished(thumbnail)))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!("Thumbnail fetch failed for article {:?}: {}", article_id, e);
                let _ = command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
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
                command_sender.send(Message::Event(Event::AsyncFetchFatArticleFinished(fat_article)))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                error!(
                    "Fat article fetch failed for article {:?}: {}",
                    article_id, e
                );
                let _ = command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
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
                let _ = command_sender.send(Message::Event(Event::AsyncOperationFailed(e.to_string())));
            }
        });
    }
}
