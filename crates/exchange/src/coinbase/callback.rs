use common::{AppInternalMessage, AppResult, SharedRwRef, Ticker};
use tokio::sync::broadcast::Sender;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use crate::{ExchangeConfig, ExchangeConfigChangeHandler};

use super::{CoinbaseChannelMessage, CoinbaseRequest, CoinbaseRequestType, CoinbaseResponse};

#[derive(Clone)]
pub struct CoinbaseWsCallback {
    client: WsClient,
    exchange_config: SharedRwRef<ExchangeConfig>,
    sender: Sender<AppInternalMessage>,
}

impl CoinbaseWsCallback {
    pub fn new(
        client: WsClient,
        exchange_config: SharedRwRef<ExchangeConfig>,
        sender: Sender<AppInternalMessage>,
    ) -> Self {
        Self {
            client,
            exchange_config,
            sender,
        }
    }

    pub fn subscribe(&self) -> AppResult<()> {
        let exchange_config = self.exchange_config.read();
        let instruments = exchange_config.get_instruments();
        let channels = exchange_config.get_channels();

        let request = CoinbaseRequest {
            request_type: CoinbaseRequestType::Subscribe,
            product_ids: instruments.iter().map(|s| s.to_string()).collect(),
            channels: channels.iter().map(|s| s.to_string()).collect(),
        };
        let json = serde_json::to_string(&request)?;
        self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        Ok(())
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<CoinbaseChannelMessage> {
        serde_json::from_str::<CoinbaseChannelMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<CoinbaseResponse> {
        serde_json::from_str::<CoinbaseResponse>(text).ok()
    }

    fn has_config_changed(&self, config: &ExchangeConfig) -> bool {
        let exchange_config = self.exchange_config.read();
        exchange_config.get_instruments() != config.get_instruments()
            || exchange_config.get_channels() != config.get_channels()
    }

    fn unsubscribe(&self, config: &ExchangeConfig) -> AppResult<()> {
        let unsubscribe_request = CoinbaseRequest {
            request_type: CoinbaseRequestType::Unsubscribe,
            product_ids: config
                .get_instruments()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            channels: config
                .get_channels()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let json = serde_json::to_string(&unsubscribe_request)?;
        self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WsCallback for CoinbaseWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.client.ws_url(), timestamp);
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
                    if let CoinbaseChannelMessage::Ticker(ticker) = channel_message {
                        let ticker: Ticker = ticker.into();
                        match self.sender.send(AppInternalMessage::Tickers(vec![ticker])) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("failed to send ticker to consumer: {:?}", e);
                            }
                        }
                    }
                } else if let Some(response) = self.try_parsing_response(&text) {
                    log::info!("received coinbase response: {:?}", response);
                } else {
                    log::warn!("received unexpected message: {:?}", text);
                }
            }

            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Coinbase connection closed: {}", reason);
                } else {
                    log::error!("Coinbase connection closed");
                }
            }

            Message::Ping(ping) => {
                self.client.write(Message::Pong(ping))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn on_disconnect(&mut self) -> AppResult<()> {
        Ok(())
    }

    fn on_heartbeat(&mut self) -> AppResult<()> {
        Ok(())
    }
}

impl ExchangeConfigChangeHandler for CoinbaseWsCallback {
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
