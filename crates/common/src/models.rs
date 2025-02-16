use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: TickerSymbol,
    pub price: Decimal,
    pub source: Source,
    #[serde(with = "crate::timestamp_with_tz_serializer")]
    pub timestamp: jiff::Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TickerSymbol {
    BTCUSD,
    ETHUSD,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Binance,
    Kraken,
    Coinbase,
    IndexerSmoothing,
    IndexerWeightedAverage,
}

impl TickerSymbol {
    pub fn from_binance_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "BTCUSDT" => Some(Self::BTCUSD),
            "ETHUSDT" => Some(Self::ETHUSD),
            "BTCUSD" => Some(Self::BTCUSD),
            "ETHUSD" => Some(Self::ETHUSD),
            _ => None,
        }
    }

    pub fn from_kraken_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "BTC/USD" => Some(Self::BTCUSD),
            "ETH/USD" => Some(Self::ETHUSD),
            _ => None,
        }
    }

    pub fn from_coinbase_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "BTC-USD" => Some(Self::BTCUSD),
            "ETH-USD" => Some(Self::ETHUSD),
            _ => None,
        }
    }
}

impl std::fmt::Display for TickerSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BTCUSD => write!(f, "BTCUSD"),
            Self::ETHUSD => write!(f, "ETHUSD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInternalMessage {
    Tickers(Vec<Ticker>),
}
