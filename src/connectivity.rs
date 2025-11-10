use std::{sync::Arc, time::Duration};

use log::{info, warn};
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

        let reachable_result = news_flash.is_reachable(&client).await;
        if !reachable_result.is_ok_and(|reachable| reachable) {
            warn!("service is not reachable anymore");
            self.message_sender
                .send(Message::Event(Event::ConnectionLost(
                    ConnectionLostReason::NotReachable,
                )))?;
        } else {
            info!("service is reachable ");
            self.message_sender
                .send(Message::Event(Event::ConnectionAvailable))?;
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
        // let is_running = self.is_running.clone();
        // let news_flash_utils = self.news_flash_utils.clone();
        // let message_sender = self.message_sender.clone();

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
