use common::{AppInternalMessage, AppResult, SharedRwRef, Ticker};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use crate::{ExchangeConfig, ExchangeConfigChangeHandler};

use super::{BinanceChannelMessage, BinanceRequest, BinanceRequestMethod, BinanceResponse};

#[derive(Clone)]
pub struct BinanceWsCallback {
    ws_client: WsClient,
    exchange_config: SharedRwRef<ExchangeConfig>,
    producer: Sender<AppInternalMessage>,
    next_request_id: u64,
}

impl BinanceWsCallback {
    pub fn new(
        ws_client: WsClient,
        exchange_config: SharedRwRef<ExchangeConfig>,
        producer: Sender<AppInternalMessage>,
    ) -> Self {
        Self {
            ws_client,
            exchange_config,
            producer,
            next_request_id: 0,
        }
    }

    pub fn subscribe(&mut self) -> AppResult<()> {
        let exchange_config = self.exchange_config.read();
        let instruments = exchange_config.get_instruments();
        let channels = exchange_config.get_channels();
        let mut params = Vec::new();
        for instrument in instruments.iter() {
            for channel in channels.iter() {
                params.push(format!(
                    "{}@{}",
                    instrument.to_lowercase(),
                    channel.to_lowercase()
                ));
            }
        }
        self.next_request_id += 1;
        let request = BinanceRequest {
            method: BinanceRequestMethod::Subscribe,
            params,
            id: self.next_request_id,
        };
        let json = serde_json::to_string(&request)?;
        self.ws_client.write(Message::Text(Utf8Bytes::from(&json)))
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<BinanceChannelMessage> {
        serde_json::from_str::<BinanceChannelMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<BinanceResponse> {
        serde_json::from_str::<BinanceResponse>(text).ok()
    }

    pub fn has_config_changed(&self, config: &ExchangeConfig) -> bool {
        let exchange_config = self.exchange_config.read();
        exchange_config.get_instruments() != config.get_instruments()
            || exchange_config.get_channels() != config.get_channels()
    }

    pub fn unsubscribe(&self, config: &ExchangeConfig) -> AppResult<()> {
        let mut params = Vec::new();
        for instrument in config.get_instruments().iter() {
            for channel in config.get_channels().iter() {
                params.push(format!(
                    "{}@{}",
                    instrument.to_lowercase(),
                    channel.to_lowercase()
                ));
            }
        }
        let request = BinanceRequest {
            method: BinanceRequestMethod::Unsubscribe,
            params,
            id: self.next_request_id,
        };
        let json = serde_json::to_string(&request)?;
        self.ws_client.write(Message::Text(Utf8Bytes::from(&json)))
    }
}

#[async_trait::async_trait]
impl WsCallback for BinanceWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.ws_client.ws_url(), timestamp);
        self.subscribe()
    }

    async fn on_message(
        &mut self,
        message: Message,
        _received_time: jiff::Timestamp,
    ) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                if let Some(channel_message) = self.try_parsing_channel_message(&text) {
                    let ticker: Ticker = channel_message.into();
                    match self
                        .producer
                        .send(AppInternalMessage::Tickers(vec![ticker]))
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("failed to send ticker to consumer: {:?}", e);
                        }
                    }
                } else if let Some(response) = self.try_parsing_response(&text) {
                    log::info!("received binance response: {:?}", response);
                } else {
                    log::warn!("received unexpected message: {:?}", text);
                }
            }
            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Binance connection closed: {}", reason);
                } else {
                    log::error!("Binance connection closed");
                }
            }
            Message::Ping(ping) => {
                self.ws_client.write(Message::Pong(ping))?;
            }
            _ => {
                log::error!("received unexpected message: {:?}", message);
            }
        }
        Ok(())
    }

    fn on_disconnect(&mut self) -> AppResult<()> {
        log::error!("binance ws connection disconnected");
        Ok(())
    }

    fn on_heartbeat(&mut self) -> AppResult<()> {
        log::debug!("binance ws heartbeat");
        Ok(())
    }
}

impl ExchangeConfigChangeHandler for BinanceWsCallback {
    fn handle_config_change(&mut self, config: ExchangeConfig) -> AppResult<()> {
        let subscription_changed = self.has_config_changed(&config);

        if subscription_changed {
            self.unsubscribe(&self.exchange_config.read())?;
            *self.exchange_config.write() = config;
            self.subscribe()?;
        }
        Ok(())
    }
}
