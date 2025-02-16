use std::collections::HashMap;

use common::SharedRwRef;
use etcd::EtcdWatcherHandler;
use exchange::{Exchange, ExchangeConfig, ExchangeConfigChangeHandler};
use serde::Deserialize;

pub type ExchangeConfigHandlerRef = Box<dyn ExchangeConfigChangeHandler + Send + Sync>;

#[derive(Clone, Debug, Deserialize)]
pub struct IndexerConfig {
    pub exchanges: Vec<ExchangeConfig>,
}

impl IndexerConfig {
    pub fn get_exchange_config(&self, exchange: Exchange) -> Option<&ExchangeConfig> {
        self.exchanges
            .iter()
            .find(|exchange_config| exchange_config.exchange == exchange)
    }
}

#[derive(Clone, Default)]
pub struct IndexerConfigChangeHandler {
    pub callback: SharedRwRef<HashMap<Exchange, ExchangeConfigHandlerRef>>,
}

impl IndexerConfigChangeHandler {
    pub fn new() -> Self {
        Self {
            callback: SharedRwRef::new(HashMap::new()),
        }
    }

    pub fn add_handler(&mut self, exchange: Exchange, handler: ExchangeConfigHandlerRef) {
        self.callback.write().insert(exchange, handler);
    }
}

impl EtcdWatcherHandler<IndexerConfig> for IndexerConfigChangeHandler {
    fn handle_config_change(&self, config: IndexerConfig) {
        for exchange_config in config.exchanges {
            if let Some(handler) = self.callback.write().get_mut(&exchange_config.exchange) {
                let _ = handler.handle_config_change(exchange_config);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use exchange::Exchange;

    use super::*;

    #[test]
    fn test_indexer_config_deserialize() {
        let config = serde_json::json!({
            "exchanges": [
                {
                    "exchange": "kraken",
                    "ws_url": "wss://ws.kraken.com",
                    "channels": ["trades", "orderbook"],
                    "instruments": ["BTC-USD", "ETH-USD"],
                    "heartbeat_millis": 3000
                },
                {
                    "exchange": "binance",
                    "ws_url": "wss://ws.binance.com",
                    "channels": ["trades", "orderbook"],
                    "instruments": ["BTC-USD", "ETH-USD"],
                    "heartbeat_millis": 3000
                },
                {
                    "exchange": "coinbase",
                    "ws_url": "wss://ws.coinbase.com",
                    "channels": ["trades", "orderbook"],
                    "instruments": ["BTC-USD", "ETH-USD"],
                    "heartbeat_millis": 3000
                }
            ]
        });

        let indexer_config: IndexerConfig = serde_json::from_value(config).unwrap();
        assert_eq!(indexer_config.exchanges.len(), 3);
        assert!(matches!(
            indexer_config.exchanges[0].exchange,
            Exchange::Kraken
        ));
        assert!(matches!(
            indexer_config.exchanges[1].exchange,
            Exchange::Binance
        ));
        assert!(matches!(
            indexer_config.exchanges[2].exchange,
            Exchange::Coinbase
        ));

        assert_eq!(indexer_config.exchanges[0].ws_url, "wss://ws.kraken.com");
        assert_eq!(indexer_config.exchanges[1].ws_url, "wss://ws.binance.com");
        assert_eq!(indexer_config.exchanges[2].ws_url, "wss://ws.coinbase.com");

        let channels = vec!["trades", "orderbook"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(indexer_config.exchanges[0].channels, channels);
        assert_eq!(indexer_config.exchanges[1].channels, channels);
        assert_eq!(indexer_config.exchanges[2].channels, channels);

        let instruments = vec!["BTC-USD", "ETH-USD"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(indexer_config.exchanges[0].instruments, instruments);
        assert_eq!(indexer_config.exchanges[1].instruments, instruments);
        assert_eq!(indexer_config.exchanges[2].instruments, instruments);

        assert_eq!(indexer_config.exchanges[0].heartbeat_millis, 3000);
        assert_eq!(indexer_config.exchanges[1].heartbeat_millis, 3000);
        assert_eq!(indexer_config.exchanges[2].heartbeat_millis, 3000);
    }
}
