use common::{AppResult, SharedRwRef};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use crate::{ExchangeConfig, ExchangeConfigChangeHandler};

use super::{KrakenMessage, KrakenRequest, KrakenRequestParams, KrakenResponse};

#[derive(Clone)]
pub struct KrakenWsCallback {
    client: WsClient,
    exchange_config: SharedRwRef<ExchangeConfig>,
}



impl KrakenWsCallback {
    pub fn new(
        client: WsClient,
        exchange_config: SharedRwRef<ExchangeConfig>,
    ) -> Self {
        Self {
            client,
            exchange_config,
        }
    }

    pub fn get_subscription_request(&mut self) -> Vec<KrakenRequest> {
        let instruments = self.exchange_config.read().get_instruments();
        let channels = self.exchange_config.read().get_channels();
        let mut requests = Vec::new();
        for instrument in instruments.iter() {
            for channel in channels.iter() {
                requests.push(KrakenRequest::Subscribe { params: KrakenRequestParams { channel: channel.clone(), symbol: vec![instrument.clone()] } });
            }
        }
        requests
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<KrakenMessage> {
        serde_json::from_str::<KrakenMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<KrakenResponse> {
        serde_json::from_str::<KrakenResponse>(text).ok()
    }
}


#[async_trait::async_trait]
impl WsCallback for KrakenWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.client.ws_url(), timestamp);
        let requests = self.get_subscription_request();
        for request in requests {
            let json = serde_json::to_string(&request)?;
            self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        }
        Ok(())
    }

    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                if let Some(channel_message) = self.try_parsing_channel_message(&text) {
                    log::info!("received kraken ticker message: {:?}", channel_message);
                } else if let Some(response) = self.try_parsing_response(&text) {
                    log::info!("received kraken response: {:?}", response);
                } else {
                    log::warn!("received unexpected message: {:?}", text);
                }
            }
            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Kraken connection closed: {}", reason);
                } else {
                    log::error!("Kraken connection closed");
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
        log::error!("kraken ws connection disconnected");
        Ok(())
    }

    fn on_heartbeat(&mut self) -> AppResult<()> {
        log::debug!("kraken ws heartbeat");
        Ok(())
    }
}


impl ExchangeConfigChangeHandler for KrakenWsCallback {
    fn handle_config_change(&self, config: ExchangeConfig) {
        *self.exchange_config.write() = config;
    }
}
