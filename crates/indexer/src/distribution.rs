use std::time::Duration;

use common::{AppError, AppInternalMessage, AppResult, Broadcaster, Context, Ticker, Worker};
use reqwest::Client;
use tokio::sync::broadcast::error::RecvError;

#[derive(Clone)]
pub struct DistributionWorker {
    context: Context,
    client: Client,
    url: String,
    receiver: Broadcaster<AppInternalMessage>,
    messages: Vec<AppInternalMessage>,
}

impl DistributionWorker {
    pub fn new(context: Context, url: String, receiver: Broadcaster<AppInternalMessage>) -> Self {
        Self {
            context,
            client: Client::new(),
            url,
            receiver,
            messages: Vec::new(),
        }
    }

    async fn send_internal_message(&self, messages: Vec<AppInternalMessage>) -> AppResult<()> {
        let mut flat_tickers: Vec<Ticker> = Vec::new();
        for message in messages {
            match message {
                AppInternalMessage::Tickers(tickers) => {
                    flat_tickers.extend(tickers.into_iter());
                }
            }
        }

        let response = self
            .client
            .post(self.url.clone())
            .json(&flat_tickers)
            .send()
            .await
            .map_err(AppError::ReqwestError)?;

        if !response.status().is_success() {
            return Err(AppError::ReqwestError(
                response.error_for_status().unwrap_err(),
            ));
        }
        Ok(())
    }
}

impl Worker for DistributionWorker {
    fn spawn(&mut self) -> common::SpawnResult {
        let mut worker = self.clone();
        let context = self.context.clone();
        let distubtion_time_interval_ms = context
            .config
            .get_int("distribution_time_interval_ms")
            .unwrap_or(5000);
        let mut app = worker.context.app.subscribe();

        tokio::spawn(async move {
            let mut message_receiver = worker.receiver.receiver();
            let mut distribution_interval =
                tokio::time::interval(Duration::from_millis(distubtion_time_interval_ms as u64));

            loop {
                tokio::select! {
                    _ = app.recv() => {
                        log::info!("{} received exit message", context.name);
                        return Ok(format!("{} received exit message", context.name));
                    }
                    _ = distribution_interval.tick() => {
                        let messages = worker.messages.drain(..).collect::<Vec<_>>();
                        log::info!("{} sending {} messages", context.name, messages.len());
                        match worker.send_internal_message(messages).await {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("error sending internal message: {}", e);
                            }
                        }
                    }
                    message = message_receiver.recv() => {
                        match message {
                            Ok(message) => {
                                worker.messages.push(message);
                            },
                            Err(RecvError::Lagged(u)) => {
                                log::warn!("{} lagged listening to internal messages : {}", context.name, u);
                            }
                            Err(e) => {
                                log::error!("error receiving internal message: {}", e);
                            }
                        }
                    }
                }
            }
        })
    }
}
