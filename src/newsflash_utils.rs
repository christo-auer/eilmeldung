use std::sync::Arc;

use news_flash::{
    NewsFlash,
    models::{ArticleID, Read},
};

use reqwest::Client;
use tokio::sync::{Mutex, RwLock, mpsc::UnboundedSender};

use crate::commands::Command;

#[derive(Clone)]
pub struct NewsFlashAsyncManager {
    pub news_flash_lock: Arc<RwLock<NewsFlash>>,
    client_lock: Arc<RwLock<Client>>,
    command_sender: UnboundedSender<Command>,

    async_operation_mutex: Arc<Mutex<()>>,
}

impl NewsFlashAsyncManager {
    pub fn new(
        news_flash: NewsFlash,
        client: Client,
        command_sender: UnboundedSender<Command>,
    ) -> Self {
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
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone(); // Clone the Arc

        tokio::spawn(async move {
            // Acquire mutex inside the spawned task
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                command_sender.send(Command::AsyncSyncStarted)?;
                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;
                let new_articles = news_flash.sync(&client, Default::default()).await?;
                command_sender.send(Command::AsyncSyncFinished(new_articles))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                let _ = command_sender.send(Command::AsyncOperationFailed(e.to_string()));
            }
            // Guard drops here, allowing next operation
        });
    }

    pub fn fetch_thumbnail(&self, article_id: ArticleID) {
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                command_sender.send(Command::AsyncFetchThumbnailStarted)?;

                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                let thumbnail = news_flash
                    .get_article_thumbnail(&article_id, &client)
                    .await?;

                command_sender.send(Command::AsyncFetchThumbnailFinished(thumbnail))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                let _ = command_sender.send(Command::AsyncOperationFailed(e.to_string()));
            }
        });
    }

    pub fn fetch_fat_article(&self, article_id: ArticleID) {
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                command_sender.send(Command::AsyncFetchFatArticleStarted)?;

                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                let fat_article = news_flash
                    .scrap_content_article(&article_id, &client, None)
                    .await?;

                command_sender.send(Command::AsyncFetchFatArticleFinished(fat_article))?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                let _ = command_sender.send(Command::AsyncOperationFailed(e.to_string()));
            }
        });
    }

    pub fn set_article_status(&self, article_ids: Vec<ArticleID>, read: Read) {
        let news_flash_lock = self.news_flash_lock.clone();
        let client_lock = self.client_lock.clone();
        let command_sender = self.command_sender.clone();
        let async_operation_mutex = self.async_operation_mutex.clone();

        tokio::spawn(async move {
            let _lock = async_operation_mutex.lock().await;

            if let Err(e) = async {
                command_sender.send(Command::AsyncMarkArticlesAsReadStarted)?;

                let news_flash = news_flash_lock.read().await;
                let client = client_lock.read().await;

                news_flash
                    .set_article_read(&article_ids, read, &client)
                    .await?;

                command_sender.send(Command::AsyncMarkArticlesAsReadFinished)?;
                Ok::<_, color_eyre::Report>(())
            }
            .await
            {
                let _ = command_sender.send(Command::AsyncOperationFailed(e.to_string()));
            }
        });
    }
}
