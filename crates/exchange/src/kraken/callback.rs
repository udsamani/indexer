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

    pub fn subscribe(&mut self) -> AppResult<()> {
        let cfg = self.exchange_config.read();
        let instruments = cfg.get_instruments();
        let channels = cfg.get_channels();
        let mut requests = Vec::new();
        for channel in channels.iter() {
            requests.push(KrakenRequest::Subscribe {
                params: KrakenRequestParams {
                    channel: channel.clone(),
                    symbol: instruments.iter().map(|s| s.to_string()).collect()
                }
            });
        }
        for request in requests {
            let json = serde_json::to_string(&request)?;
            self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        }
        Ok(())
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<KrakenMessage> {
        serde_json::from_str::<KrakenMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<KrakenResponse> {
        serde_json::from_str::<KrakenResponse>(text).ok()
    }

    fn has_config_changed(&self, config: &ExchangeConfig) -> bool {
        let exchange_config = self.exchange_config.read();
        exchange_config.get_instruments() != config.get_instruments() ||
            exchange_config.get_channels() != config.get_channels()
    }

    pub fn unsubscribe(&self, config: &ExchangeConfig) -> AppResult<()> {
        let mut requests = Vec::new();
        for channel in config.get_channels().iter() {
            requests.push(KrakenRequest::Unsubscribe {
                params: KrakenRequestParams {
                    channel: channel.clone(),
                    symbol: config.get_instruments().iter().map(|s| s.to_string()).collect()
                }
            });
        }
        for request in requests {
            let json = serde_json::to_string(&request)?;
            self.client.write(Message::Text(Utf8Bytes::from(&json)))?;
        }
        Ok(())
    }
}


#[async_trait::async_trait]
impl WsCallback for KrakenWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.client.ws_url(), timestamp);
        self.subscribe()
    }

    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                if let Some(channel_message) = self.try_parsing_channel_message(&text) {
                    log::debug!("received kraken ticker message: {:?}", channel_message);
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
