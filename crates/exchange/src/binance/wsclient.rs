use common::{AppInternalMessage, Context, SharedRwRef};
use tokio::sync::mpsc::Sender;
use wsclient::{WsClient, WsConsumer};

use crate::ExchangeConfig;

use super::BinanceWsCallback;

#[derive(Clone)]
pub struct BinanceWsClient {
    client: WsClient,
    config: SharedRwRef<ExchangeConfig>,
}

impl BinanceWsClient {
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
    ) -> WsConsumer<BinanceWsCallback> {
        let callback = BinanceWsCallback::new(self.client.clone(), self.config.clone(), sender);
        self.client
            .consumer(context.with_name("binance-ws-consumer"), callback)
    }
}
