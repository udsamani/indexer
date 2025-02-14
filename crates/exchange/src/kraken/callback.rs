use std::collections::HashSet;

use common::{AppResult, SharedRwRef};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use super::{KrakenMessage, KrakenRequest, KrakenRequestParams, KrakenResponse};

#[derive(Clone)]
pub struct KrakenWsCallback {
    client: WsClient,
    instruments_to_subscribe: SharedRwRef<HashSet<String>>,
    channel_to_subscribe: SharedRwRef<HashSet<String>>,
}



impl KrakenWsCallback {
    pub fn new(
        client: WsClient,
        instruments_to_subscribe: HashSet<String>,
        channel_to_subscribe: HashSet<String>,
    ) -> Self {
        Self {
            client,
            instruments_to_subscribe: SharedRwRef::new(instruments_to_subscribe),
            channel_to_subscribe: SharedRwRef::new(channel_to_subscribe),
        }
    }

    pub fn get_subscription_request(&mut self) -> Vec<KrakenRequest> {
        let instruments = self.instruments_to_subscribe.read().clone();
        let channels = self.channel_to_subscribe.read().clone();
        let mut requests = Vec::new();
        for instrument in instruments.iter() {
            for channel in channels.iter() {
                requests.push(KrakenRequest::Subscribe { params: KrakenRequestParams { channel: channel.clone(), symbol: vec![instrument.clone()] } });
            }
        }
        requests
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
                let message_result = serde_json::from_str::<KrakenMessage>(&text);
                match message_result {
                    Ok(channel_message) => {
                        // Unfortunately kraken does not provide instrument at top message level, this
                        // is bit inconvient, given the fact that we may be in this state where the config
                        // has changed, and we are not longer interested in the ticker for the old instruments.
                        // TODO: handle this better, might be good idea to create a map of symbol -> messages
                        if let KrakenMessage::ChannelMessage(channel_message) = channel_message {
                            if self.channel_to_subscribe.read().contains(&channel_message.channel.to_string()) {
                                log::info!("received kraken ticker message: {:?}", channel_message);
                            }
                        }
                    }
                    Err(_) => {
                        let response_result = serde_json::from_str::<KrakenResponse>(&text);
                        match response_result {
                            Ok(response) => {
                                log::info!("received kraken response: {:?}", response);
                            }
                            Err(e) => {
                                log::error!("error parsing kraken response: {}", e);
                            }
                        }
                    }
                }
            }
            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Kraken connection closed: {}", reason);
                } else {
                    log::error!("Kraken connection closed");
                }
            }
            _ => {
                log::error!("received unexpected message: {:?}", message);
            }
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
