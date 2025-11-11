use std::{sync::Arc, time::Duration};

use log::{info, warn};
use news_flash::error::{FeedApiError, NewsFlashError};
use reqwest::Client;
use tokio::task::JoinHandle;

use crate::prelude::*;
use network_connectivity::{Connectivity, ConnectivityState};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

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
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        let is_reachable = news_flash.is_reachable(&client).await;
        match is_reachable {
            // if reachable or not supported => reachable
            Ok(true) | Err(NewsFlashError::API(FeedApiError::Unsupported)) => {
                info!("service is reachable ");
                self.message_sender
                    .send(Message::Event(Event::ConnectionAvailable))?;
            }

            Ok(false) | Err(_) => {
                warn!("service is not reachable anymore");
                self.message_sender
                    .send(Message::Event(Event::ConnectionLost(
                        ConnectionLostReason::NotReachable,
                    )))?;
            }
        }

        Ok(())
    }

    async fn on_connectivity_changed(&self, connectivity: &Connectivity) -> color_eyre::Result<()> {
        use ConnectivityState::*;
        match (connectivity.ipv4, connectivity.ipv6) {
            (None, None) => {
                warn!("connection to the network is gone");
                self.message_sender
                    .send(Message::Event(Event::ConnectionLost(
                        ConnectionLostReason::NoInternet,
                    )))?;
            }
            (_, _) => self.check_reachability().await?,
        }

        Ok(())
    }

    pub fn spawn(self) -> color_eyre::Result<JoinHandle<color_eyre::Result<()>>> {
        let (driver, mut receiver) =
            network_connectivity::new().map_err(|error| color_eyre::eyre::eyre!(error))?;
        let driver = tokio::spawn(driver);

        Ok(tokio::spawn(async move {
            *self.is_running.lock().await = true;
            while *self.is_running.lock().await {
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        self.check_reachability().await?;
                    },
                    connectivity = receiver.recv() => {
                        match connectivity {
                            None => *self.is_running.lock().await = false,
                            Some(connectivity) => {
                                log::info!("connectivity has changed: {:?}", connectivity);
                                self.on_connectivity_changed(&connectivity).await?;
                            }

                        }

                    }


                }
            }
            drop(receiver);
            driver
                .await?
                .map_err(|error| color_eyre::eyre::eyre!(error))?;

            Ok(())
        }))
    }
}
