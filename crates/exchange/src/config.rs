use std::collections::HashSet;

use common::{AppResult, Source};
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
/// let channels = vec!["trades", "orderbook"].into_iter().map(|s| s.to_string()).collect();
/// let instruments = vec!["BTC-USD", "ETH-USD"].into_iter().map(|s| s.to_string()).collect();
///
/// let config = ExchangeConfig {
///     ws_url: "wss://ws.exchange.com/socket".to_string(),
///     channels,
///     instruments,
///     heartbeat_millis: 30000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub ws_url: String,
    pub channels: HashSet<String>,
    pub instruments: HashSet<String>,
    pub heartbeat_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    Kraken,
    Coinbase,
    Binance,
}

impl Exchange {
    pub fn from_source(source: &Source) -> Option<Self> {
        match source {
            Source::Binance => Some(Self::Binance),
            Source::Kraken => Some(Self::Kraken),
            Source::Coinbase => Some(Self::Coinbase),
            _ => None,
        }
    }
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Kraken => write!(f, "kraken"),
            Exchange::Coinbase => write!(f, "coinbase"),
            Exchange::Binance => write!(f, "binance"),
        }
    }
}

impl ExchangeConfig {
    pub fn new(
        ws_url: String,
        channels: HashSet<String>,
        instruments: HashSet<String>,
        heartbeat_millis: u64,
    ) -> Self {
        Self {
            ws_url,
            channels,
            instruments,
            heartbeat_millis,
        }
    }

    pub fn get_instruments(&self) -> &HashSet<String> {
        &self.instruments
    }

    pub fn get_channels(&self) -> &HashSet<String> {
        &self.channels
    }
}

pub trait ExchangeConfigChangeHandler {
    fn handle_config_change(&mut self, config: ExchangeConfig) -> AppResult<()>;
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
        let channels = vec!["trades", "orderbook"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
        let instruments = vec!["BTC-USD", "ETH-USD"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
        assert_eq!(config.ws_url, "wss://ws.exchange.com/socket");
        assert_eq!(config.channels, channels);
        assert_eq!(config.instruments, instruments);
        assert_eq!(config.heartbeat_millis, 30000);
    }
}
