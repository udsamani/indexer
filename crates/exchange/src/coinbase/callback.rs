use common::{AppResult, SharedRwRef};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use crate::{ExchangeConfig, ExchangeConfigChangeHandler};

use super::{CoinbaseChannelMessage, CoinbaseRequest, CoinbaseRequestType, CoinbaseResponse};


#[derive(Clone)]
pub struct CoinbaseWsCallback {
    client: WsClient,
    exchange_config: SharedRwRef<ExchangeConfig>,
}

impl CoinbaseWsCallback {
    pub fn new(client: WsClient, exchange_config: SharedRwRef<ExchangeConfig>) -> Self {
        Self { client, exchange_config }
    }

    pub fn get_subscription_request(&mut self) -> CoinbaseRequest {
        let instruments = self.exchange_config.read().get_instruments();
        let channels = self.exchange_config.read().get_channels();

        CoinbaseRequest {
            request_type: CoinbaseRequestType::Subscribe,
            product_ids: instruments.into_iter().collect(),
            channels: channels.into_iter().collect(),
        }
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<CoinbaseChannelMessage> {
        serde_json::from_str::<CoinbaseChannelMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<CoinbaseResponse> {
        serde_json::from_str::<CoinbaseResponse>(text).ok()
    }
}


#[async_trait::async_trait]
impl WsCallback for CoinbaseWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.client.ws_url(), timestamp);
        let request = self.get_subscription_request();
        let json = serde_json::to_string(&request)?;
        self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        Ok(())
    }

    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                if let Some(channel_message) = self.try_parsing_channel_message(&text) {
                    log::info!("received coinbase ticker message: {:?}", channel_message);
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
    fn handle_config_change(&self, config: ExchangeConfig) {
        *self.exchange_config.write() = config;
    }
}
