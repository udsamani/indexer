use std::collections::HashSet;

use common::{AppResult, SharedRwRef};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use super::{BinanceChannelMessage, BinanceRequest, BinanceRequestMethod, BinanceResponse};


#[derive(Clone)]
pub struct BinanceWsCallback {
    ws_client: WsClient,
    instruments_to_subscribe: SharedRwRef<HashSet<String>>,
    channel_to_subscribe: SharedRwRef<HashSet<String>>,
    next_request_id: u64,
}


impl BinanceWsCallback {

    pub fn new(ws_client: WsClient, instruments_to_subscribe: HashSet<String>, channel_to_subscribe: HashSet<String>) -> Self {
        Self {
            ws_client,
            instruments_to_subscribe: SharedRwRef::new(instruments_to_subscribe),
            channel_to_subscribe: SharedRwRef::new(channel_to_subscribe),
            next_request_id: 0,
        }
    }

    pub fn get_subscription_request(&mut self) -> BinanceRequest {
        let instruments = self.instruments_to_subscribe.read();
        let channels = self.channel_to_subscribe.read();
        let mut params = Vec::new();
        for instrument in instruments.iter() {
            for channel in channels.iter() {
                params.push(format!("{}@{}", instrument.to_lowercase(), channel.to_lowercase()));
            }
        }
        self.next_request_id += 1;
        BinanceRequest {
            method: BinanceRequestMethod::Subscribe,
            params,
            id: self.next_request_id,
        }
    }

    pub fn try_parsing_channel_message(&self, text: &Utf8Bytes) -> Option<BinanceChannelMessage> {
        serde_json::from_str::<BinanceChannelMessage>(text).ok()
    }

    pub fn try_parsing_response(&self, text: &Utf8Bytes) -> Option<BinanceResponse> {
        serde_json::from_str::<BinanceResponse>(text).ok()
    }

}


#[async_trait::async_trait]
impl WsCallback for BinanceWsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()> {
        log::info!("connected to {} at {}", self.ws_client.ws_url(), timestamp);
        let request = self.get_subscription_request();
        let json = serde_json::to_string(&request)?;
        self.ws_client.write(Message::Text(Utf8Bytes::from(&json)))
    }

    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> AppResult<()> {
        match message {
            Message::Text(text) => {
                if let Some(channel_message) = self.try_parsing_channel_message(&text) {
                    log::info!("received binance ticker message: {:?}", channel_message);
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
