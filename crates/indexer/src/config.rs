use std::collections::HashMap;

use common::SharedRwRef;
use etcd::EtcdWatcherHandler;
use exchange::{Exchange, ExchangeConfig, ExchangeConfigChangeHandler};
use serde::Deserialize;

use crate::processing::{SmoothingConfig, SmoothingConfigChangeHandler};

pub type ExchangeConfigHandlerRef = Box<dyn ExchangeConfigChangeHandler + Send + Sync>;
pub type SmoothingConfigChangeHandlerRef = Box<dyn SmoothingConfigChangeHandler + Send + Sync>;

#[derive(Clone, Debug, Deserialize)]
pub struct IndexerConfig {
    #[serde(flatten)]
    config: HashMap<Exchange, FeedConfig>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct FeedConfig {
    exchange_config: ExchangeConfig,
    smoothing_config: SmoothingConfig,
}

#[allow(unused)]
impl IndexerConfig {
    pub fn get_exchange_config(&self, exchange: Exchange) -> Option<&ExchangeConfig> {
        self.config
            .get(&exchange)
            .map(|feed_config| &feed_config.exchange_config)
    }

    pub fn get_smoothing_config(&self, exchange: Exchange) -> Option<&SmoothingConfig> {
        self.config
            .get(&exchange)
            .map(|feed_config| &feed_config.smoothing_config)
    }
}

#[derive(Clone, Default)]
pub struct IndexerConfigChangeHandler {
    exchange_config_callbacks: SharedRwRef<HashMap<Exchange, ExchangeConfigHandlerRef>>,
    smoothing_config_callbacks: SharedRwRef<HashMap<Exchange, SmoothingConfigChangeHandlerRef>>,
}

impl IndexerConfigChangeHandler {
    pub fn new() -> Self {
        Self {
            exchange_config_callbacks: SharedRwRef::new(HashMap::new()),
            smoothing_config_callbacks: SharedRwRef::new(HashMap::new()),
        }
    }

    pub fn add_exchange_config_handler(
        &mut self,
        exchange: Exchange,
        handler: ExchangeConfigHandlerRef,
    ) {
        self.exchange_config_callbacks
            .write()
            .insert(exchange, handler);
    }

    pub fn add_smoothing_config_handler(
        &mut self,
        exchange: Exchange,
        handler: SmoothingConfigChangeHandlerRef,
    ) {
        self.smoothing_config_callbacks
            .write()
            .insert(exchange, handler);
    }
}

impl EtcdWatcherHandler<IndexerConfig> for IndexerConfigChangeHandler {
    fn handle_config_change(&self, config: IndexerConfig) {
        for (exchange, feed_config) in config.config {
            if let Some(handler) = self.exchange_config_callbacks.write().get_mut(&exchange) {
                let _ = handler.handle_config_change(feed_config.exchange_config);
            }
            if let Some(handler) = self.smoothing_config_callbacks.write().get_mut(&exchange) {
                let _ = handler.handle_config_change(&exchange, feed_config.smoothing_config);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use exchange::Exchange;
    use rust_decimal_macros::dec;

    use crate::processing::SmoothingConfig;

    use super::IndexerConfig;

    #[test]
    fn test_indexer_config_deserialize() {
        let config = serde_json::json!({
            "kraken": {
                "exchange_config": {
                    "ws_url": "wss://ws.kraken.com/v2",
                    "channels": ["ticker"],
                    "instruments": ["BTC/USD", "ETH/USD"],
                    "heartbeat_millis": 3000
                },
                "smoothing_config": {
                    "type": "ema",
                    "params": {
                        "window": 100,
                        "smoothing": 2.0
                    }
                }
            },
            "binance": {
                "exchange_config": {
                    "ws_url": "wss://stream.binance.com:9443/ws",
                    "channels": ["ticker"],
                    "instruments": ["ETHUSDT", "BTCUSDT"],
                    "heartbeat_millis": 3000
                },
                "smoothing_config": {
                    "type": "sma",
                    "params": {
                        "window": 100
                    }
                }
            },
            "coinbase": {
                "exchange_config": {
                    "ws_url": "wss://ws-feed.exchange.coinbase.com",
                    "channels": ["ticker", "heartbeat"],
                    "instruments": ["BTC-USD", "ETH-USD"],
                    "heartbeat_millis": 3000
                },
                "smoothing_config": {
                    "type": "sma",
                    "params": {
                        "window": 100
                    }
                }
            }
        });

        let indexer_config: IndexerConfig = serde_json::from_value(config).unwrap();
        let instruments = vec!["BTC/USD", "ETH/USD"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let channels = vec!["ticker"].into_iter().map(|s| s.to_string()).collect();

        let kraken_config = indexer_config
            .get_exchange_config(Exchange::Kraken)
            .unwrap();

        assert_eq!(kraken_config.ws_url, "wss://ws.kraken.com/v2");
        assert_eq!(kraken_config.channels, channels);
        assert_eq!(kraken_config.instruments, instruments);
        assert_eq!(kraken_config.heartbeat_millis, 3000);

        let kraken_smoothing_config = indexer_config
            .get_smoothing_config(Exchange::Kraken)
            .unwrap();
        match kraken_smoothing_config {
            SmoothingConfig::ExponentialMovingAverage { params } => {
                assert_eq!(params.window, 100);
                assert_eq!(params.smoothing, dec!(2.0));
                assert_eq!(
                    params.smoothing_factor(),
                    dec!(0.0198019801980198019801980198)
                );
            }
            _ => panic!("Expected ExponentialMovingAverage"),
        }

        let instruments = vec!["ETHUSDT", "BTCUSDT"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let channels = vec!["ticker"].into_iter().map(|s| s.to_string()).collect();

        let binance_config = indexer_config
            .get_exchange_config(Exchange::Binance)
            .unwrap();
        assert_eq!(binance_config.ws_url, "wss://stream.binance.com:9443/ws");
        assert_eq!(binance_config.channels, channels);
        assert_eq!(binance_config.instruments, instruments);
        assert_eq!(binance_config.heartbeat_millis, 3000);

        let binance_smoothing_config = indexer_config
            .get_smoothing_config(Exchange::Binance)
            .unwrap();
        match binance_smoothing_config {
            SmoothingConfig::SimpleMovingAverage { params } => {
                assert_eq!(params.window, 100);
            }
            _ => panic!("Expected SimpleMovingAverage"),
        }

        let instruments = vec!["BTC-USD", "ETH-USD"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let channels = vec!["ticker", "heartbeat"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let coinbase_config = indexer_config
            .get_exchange_config(Exchange::Coinbase)
            .unwrap();
        assert_eq!(
            coinbase_config.ws_url,
            "wss://ws-feed.exchange.coinbase.com"
        );
        assert_eq!(coinbase_config.channels, channels);
        assert_eq!(coinbase_config.instruments, instruments);
        assert_eq!(coinbase_config.heartbeat_millis, 3000);

        let coinbase_smoothing_config = indexer_config
            .get_smoothing_config(Exchange::Coinbase)
            .unwrap();
        match coinbase_smoothing_config {
            SmoothingConfig::SimpleMovingAverage { params } => {
                assert_eq!(params.window, 100);
            }
            _ => panic!("Expected SimpleMovingAverage"),
        }
    }
}
