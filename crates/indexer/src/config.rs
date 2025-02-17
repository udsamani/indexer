use std::collections::HashMap;

use common::{Context, SharedRwRef};
use etcd::EtcdWatcherHandler;
use exchange::{Exchange, ExchangeConfig, ExchangeConfigChangeHandler};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::processing::{
    SmoothingConfig, SmoothingConfigChangeHandler, WeightedAverageConfig,
    WeightedAverageConfigChangeHandler,
};

pub type ExchangeConfigHandlerRef = Box<dyn ExchangeConfigChangeHandler + Send + Sync>;
pub type SmoothingConfigChangeHandlerRef = Box<dyn SmoothingConfigChangeHandler + Send + Sync>;
pub type WeightedAverageConfigChangeHandlerRef =
    Box<dyn WeightedAverageConfigChangeHandler + Send + Sync>;

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
    weight: Decimal,
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

    pub fn get_weight(&self, exchange: Exchange) -> Option<&Decimal> {
        self.config
            .get(&exchange)
            .map(|feed_config| &feed_config.weight)
    }
}

#[derive(Clone)]
pub struct IndexerConfigChangeHandler {
    context: Context,
    exchange_config_callbacks: SharedRwRef<HashMap<Exchange, ExchangeConfigHandlerRef>>,
    smoothing_config_callbacks: SharedRwRef<HashMap<Exchange, SmoothingConfigChangeHandlerRef>>,
    weighted_average_config_callbacks: SharedRwRef<Vec<WeightedAverageConfigChangeHandlerRef>>,
}

impl IndexerConfigChangeHandler {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            exchange_config_callbacks: SharedRwRef::new(HashMap::new()),
            smoothing_config_callbacks: SharedRwRef::new(HashMap::new()),
            weighted_average_config_callbacks: SharedRwRef::new(Vec::new()),
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

    pub fn add_weighted_average_config_handler(
        &mut self,
        handler: WeightedAverageConfigChangeHandlerRef,
    ) {
        self.weighted_average_config_callbacks.write().push(handler);
    }
}

impl EtcdWatcherHandler<IndexerConfig> for IndexerConfigChangeHandler {
    fn handle_config_change(&self, config: IndexerConfig) {
        let mut weights = HashMap::new();
        for (exchange, feed_config) in &config.config {
            if let Some(handler) = self.exchange_config_callbacks.write().get_mut(exchange) {
                match handler.handle_config_change(feed_config.exchange_config.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("error handling exchange config change: {}", e);
                        let _ = self
                            .context
                            .log_and_exit(&format!("error handling exchange config change: {}", e))
                            .unwrap();
                    }
                }
            }
            if let Some(handler) = self.smoothing_config_callbacks.write().get_mut(exchange) {
                match handler.handle_config_change(exchange, feed_config.smoothing_config.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("error handling smoothing config change: {}", e);
                        let _ = self
                            .context
                            .log_and_exit(&format!("error handling smoothing config change: {}", e))
                            .unwrap();
                    }
                }
            }
            if !feed_config.exchange_config.instruments.is_empty() {
                weights.insert(exchange.clone(), feed_config.weight);
            }
        }

        let weighted_average_config = WeightedAverageConfig::new(weights);
        if let Err(e) = &weighted_average_config {
            log::error!("error creating weighted average config: {}", e);
            let _ = self
                .context
                .log_and_exit(&format!("error creating weighted average config: {}", e))
                .unwrap();
            return;
        }

        let weighted_average_config = weighted_average_config.unwrap();
        for handler in self.weighted_average_config_callbacks.write().iter_mut() {
            match handler.handle_config_change(weighted_average_config.clone()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("error handling weighted average config change: {}", e);
                    let _ = self
                        .context
                        .log_and_exit(&format!(
                            "error handling weighted average config change: {}",
                            e
                        ))
                        .unwrap();
                }
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
                },
                "weight": 30.0
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
                },
                "weight": 40.0
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
                },
                "weight": 30.0
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

        assert_eq!(
            indexer_config.get_weight(Exchange::Kraken),
            Some(&dec!(30.0))
        );

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

        assert_eq!(
            indexer_config.get_weight(Exchange::Binance),
            Some(&dec!(40.0))
        );

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

        assert_eq!(
            indexer_config.get_weight(Exchange::Coinbase),
            Some(&dec!(30.0))
        );
    }
}
