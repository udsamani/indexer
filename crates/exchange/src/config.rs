use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Configuration for establishing and maintaining a WebSocket connection to a cryptocurrency exchange.
///
/// # Fields
/// - `ws_url`: WebSocket endpoint URL for the exchange connection
/// - `channels`: List of data feed channels to subscribe to (e.g., trades, orderbook, ticker)
/// - `instruments`: Trading pairs to monitor (e.g., BTC-USD, ETH-USD)
/// - `heartbeat_millis`: Heartbeat interval in milliseconds
///
/// # Example
/// ```
/// use exchange::ExchangeConfig;
/// use exchange::Exchange;
///
/// let config = ExchangeConfig {
///     exchange: Exchange::Kraken,
///     ws_url: "wss://ws.exchange.com/socket".to_string(),
///     channels: vec!["trades".to_string(), "orderbook".to_string()],
///     instruments: vec!["BTC-USD".to_string()],
///     heartbeat_millis: 30000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub ws_url: String,
    pub exchange: Exchange,
    pub channels: Vec<String>,
    pub instruments: Vec<String>,
    pub heartbeat_millis: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    Kraken,
    Coinbase,
    Binance,
}

impl ExchangeConfig {
    pub fn new(
        ws_url: String,
        exchange: Exchange,
        channels: Vec<String>,
        instruments: Vec<String>,
        heartbeat_millis: u64,
    ) -> Self {
        Self { ws_url, exchange, channels, instruments, heartbeat_millis }
    }

    pub fn get_instruments(&self) -> HashSet<String> {
        self.instruments.clone().into_iter().collect()
    }

    pub fn get_channels(&self) -> HashSet<String> {
        self.channels.clone().into_iter().collect()
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_config_deserialize() {
        let config_json = serde_json::json!({
            "ws_url": "wss://ws.exchange.com/socket",
            "exchange": "kraken",
            "channels": ["trades", "orderbook"],
            "instruments": ["BTC-USD", "ETH-USD"],
            "heartbeat_millis": 30000
        });

        let config: ExchangeConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.ws_url, "wss://ws.exchange.com/socket");
        assert!(matches!(config.exchange, Exchange::Kraken));
        assert_eq!(config.channels, vec!["trades", "orderbook"]);
        assert_eq!(config.instruments, vec!["BTC-USD", "ETH-USD"]);
        assert_eq!(config.heartbeat_millis, 30000);
    }
}
