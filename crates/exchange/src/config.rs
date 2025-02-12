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
///
/// let config = ExchangeConfig {
///     ws_url: "wss://ws.exchange.com/socket".to_string(),
///     channels: vec!["trades".to_string(), "orderbook".to_string()],
///     instruments: vec!["BTC-USD".to_string()],
///     heartbeat_millis: 30000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub ws_url: String,
    pub channels: Vec<String>,
    pub instruments: Vec<String>,
    pub heartbeat_millis: u64,
}

impl ExchangeConfig {
    pub fn new(
        ws_url: String,
        channels: Vec<String>,
        instruments: Vec<String>,
        heartbeat_millis: u64,
    ) -> Self {
        Self { ws_url, channels, instruments, heartbeat_millis }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_config_deserialize() {
        let config_json = serde_json::json!({
            "ws_url": "wss://ws.exchange.com/socket",
            "channels": ["trades", "orderbook"],
            "instruments": ["BTC-USD", "ETH-USD"],
            "heartbeat_millis": 30000
        });

        let config: ExchangeConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.ws_url, "wss://ws.exchange.com/socket");
        assert_eq!(config.channels, vec!["trades", "orderbook"]);
        assert_eq!(config.instruments, vec!["BTC-USD", "ETH-USD"]);
        assert_eq!(config.heartbeat_millis, 30000);
    }

    #[test]
    fn test_exchange_config_serialize() {
        let config = ExchangeConfig::new(
            "wss://ws.exchange.com/socket".to_string(),
            vec!["trades".to_string(), "orderbook".to_string()],
            vec!["BTC-USD".to_string(), "ETH-USD".to_string()],
            30000
        );

        let config_json = serde_json::to_value(&config).unwrap();
        assert_eq!(config_json, config_json);
    }
}
