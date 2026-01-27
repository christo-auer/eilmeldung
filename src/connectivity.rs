use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use futures::StreamExt;
use log::{info, trace, warn};
use news_flash::error::{FeedApiError, NewsFlashError};
use tokio::task::JoinHandle;

use crate::prelude::*;
use tokio::sync::{Mutex, mpsc::UnboundedSender};

const IS_REACHABLE_RETRIES: u16 = 10;
const TIME_BETWEEN_RETRIES: Duration = Duration::from_secs(1);

pub struct ConnectivityMonitor {
    message_sender: UnboundedSender<Message>,
    news_flash_utils: Arc<NewsFlashUtils>,
    is_running: Arc<Mutex<bool>>,
}

#[derive(Debug)]
pub enum ConnectionLostReason {
    NoInternet,
    NotReachable,
}

impl ConnectivityMonitor {
    pub fn new(
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            message_sender,
            news_flash_utils,

            is_running: Arc::new(Mutex::new(false)),
        }
    }

    async fn check_reachability(&self) -> color_eyre::Result<()> {
        let news_flash = self.news_flash_utils.news_flash_lock.read().await;
        info!("checking reachability of service");
        let client = build_client(Duration::from_secs(10))?;

        let mut is_reachable: bool = true;

        for _ in 0..IS_REACHABLE_RETRIES {
            trace!("polling service for reachability");
            let reachable_result = news_flash.is_reachable(&client).await;

            // Only consider it unreachable on actual network errors
            match &reachable_result {
                Err(NewsFlashError::API(FeedApiError::Network(_))) => {
                    is_reachable = false;
                    sleep(TIME_BETWEEN_RETRIES).await;
                }
                _ => {
                    // Ok(true), Ok(false), Unsupported, Login, Auth, etc. - not a connectivity issue
                    // Ok(false) can happen when the server returns non-2xx for HEAD requests,
                    // which doesn't mean we're offline
                    is_reachable = true;
                    break;
                }
            }
        }

        match is_reachable {
            // if reachable or not supported => reachable
            true => {
                info!("service is reachable ");
                self.message_sender
                    .send(Message::Event(Event::ConnectionAvailable))?;
            }

            false => {
                warn!("service is not reachable anymore");
                self.message_sender
                    .send(Message::Event(Event::ConnectionLost(
                        ConnectionLostReason::NotReachable,
                    )))?;
            }
        }

        Ok(())
    }

    pub fn spawn(self) -> color_eyre::Result<JoinHandle<color_eyre::Result<()>>> {
        use if_watch::tokio::IfWatcher;

        let mut set = IfWatcher::new()?;

        Ok(tokio::spawn(async move {
            *self.is_running.lock().await = true;
            while *self.is_running.lock().await {
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        self.check_reachability().await?;
                    },

                    event = set.select_next_some() => {
                        log::info!("connectivity has changed: {:?}", event);
                        self.check_reachability().await?;
                    }


                }
            }

            Ok(())
        }))
    }
}
