use common::{AppInternalMessage, Context, SharedRwRef};
use tokio::sync::broadcast::Sender;
use wsclient::{WsClient, WsConsumer};

use crate::ExchangeConfig;

use super::CoinbaseWsCallback;

#[derive(Clone)]
pub struct CoinbaseWsClient {
    ws_client: WsClient,
    config: SharedRwRef<ExchangeConfig>,
}

impl CoinbaseWsClient {
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            ws_client: WsClient::new(config.ws_url.clone(), config.heartbeat_millis),
            config: SharedRwRef::new(config),
        }
    }

    pub fn consumer(
        &mut self,
        context: Context,
        sender: Sender<AppInternalMessage>,
    ) -> WsConsumer<CoinbaseWsCallback> {
        let callback = CoinbaseWsCallback::new(self.ws_client.clone(), self.config.clone(), sender);
        self.ws_client
            .consumer(context.with_name("coinbase-ws-consumer"), callback)
    }
}
