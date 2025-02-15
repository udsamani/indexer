use common::{AppInternalMessage, Context, SharedRwRef};
use tokio::sync::mpsc::Sender;
use wsclient::{WsClient, WsConsumer};

use crate::ExchangeConfig;

use super::KrakenWsCallback;

#[derive(Clone)]
pub struct KrakenWsClient {
    client: WsClient,
    config: SharedRwRef<ExchangeConfig>,
}

impl KrakenWsClient {
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            client: WsClient::new(config.ws_url.clone(), config.heartbeat_millis),
            config: SharedRwRef::new(config),
        }
    }

    pub fn consumer(
        &mut self,
        context: Context,
        sender: Sender<AppInternalMessage>,
    ) -> WsConsumer<KrakenWsCallback> {
        let callback = KrakenWsCallback::new(self.client.clone(), self.config.clone(), sender);
        self.client
            .consumer(context.with_name("kraken-ws-consumer"), callback)
    }
}
