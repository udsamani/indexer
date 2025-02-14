use exchange::ExchangeConfig;
use serde::Deserialize;


#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct IndexerConfig {
    pub exchanges: Vec<ExchangeConfig>,
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
        assert!(matches!(indexer_config.exchanges[0].exchange, Exchange::Kraken));
        assert!(matches!(indexer_config.exchanges[1].exchange, Exchange::Binance));
        assert!(matches!(indexer_config.exchanges[2].exchange, Exchange::Coinbase));

        assert_eq!(indexer_config.exchanges[0].ws_url, "wss://ws.kraken.com");
        assert_eq!(indexer_config.exchanges[1].ws_url, "wss://ws.binance.com");
        assert_eq!(indexer_config.exchanges[2].ws_url, "wss://ws.coinbase.com");

        assert_eq!(indexer_config.exchanges[0].channels, vec!["trades", "orderbook"]);
        assert_eq!(indexer_config.exchanges[1].channels, vec!["trades", "orderbook"]);
        assert_eq!(indexer_config.exchanges[2].channels, vec!["trades", "orderbook"]);

        assert_eq!(indexer_config.exchanges[0].instruments, vec!["BTC-USD", "ETH-USD"]);
        assert_eq!(indexer_config.exchanges[1].instruments, vec!["BTC-USD", "ETH-USD"]);
        assert_eq!(indexer_config.exchanges[2].instruments, vec!["BTC-USD", "ETH-USD"]);

        assert_eq!(indexer_config.exchanges[0].heartbeat_millis, 3000);
        assert_eq!(indexer_config.exchanges[1].heartbeat_millis, 3000);
        assert_eq!(indexer_config.exchanges[2].heartbeat_millis, 3000);



    }
}
