use common::{Source, Ticker, TickerSymbol};
use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum KrakenRequest {
    Subscribe { params: KrakenRequestParams },
    Unsubscribe { params: KrakenRequestParams },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenRequestParams {
    pub channel: String,
    pub symbol: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenResponse {
    #[serde(flatten)]
    pub response_data: KrakenResponseData,
    pub success: bool,
    #[serde(with = "common::timestamp_with_tz_serializer")]
    pub time_in: Timestamp,
    #[serde(with = "common::timestamp_with_tz_serializer")]
    pub time_out: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum KrakenResponseData {
    Subscribe { result: KrakenResponseResult },
    Unsubscribe { result: KrakenResponseResult },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum KrakenMessage {
    ChannelMessage(KrakenChannelMessage),
    Heartbeat(KrakenHeartbeat),
}

impl KrakenMessage {
    pub fn get_tickers_internal(&self) -> Vec<Ticker> {
        let mut internal_tickers = Vec::new();
        if let KrakenMessage::ChannelMessage(channel_message) = self {
            if let KrakenChannelData::Ticker(tickers) = &channel_message.data {
                internal_tickers.extend(tickers.iter().map(|ticker| ticker.into()));
            }
        }
        internal_tickers
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KrakenChannelMessage {
    #[serde(rename = "type")]
    pub data_type: KrakenDataType,
    pub channel: KrakenChannelType,
    pub data: KrakenChannelData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenHeartbeat {
    pub channel: KrakenChannelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KrakenChannelType {
    Ticker,
    Heartbeat,
    Status,
}

impl std::fmt::Display for KrakenChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KrakenChannelType::Ticker => write!(f, "ticker"),
            KrakenChannelType::Heartbeat => write!(f, "heartbeat"),
            KrakenChannelType::Status => write!(f, "status"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KrakenDataType {
    Snapshot,
    Update,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum KrakenChannelData {
    Ticker(Vec<KrakenTicker>),
    Connection(Vec<KrakenConnection>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenTicker {
    pub ask: Decimal,
    pub bid: Decimal,
    pub symbol: String,
    pub ask_qty: Decimal,
    pub bid_qty: Decimal,
    pub last: Decimal,
    pub volume: Decimal,
    pub vwap: Decimal,
    pub low: Decimal,
    pub high: Decimal,
    pub change: Decimal,
    pub change_pct: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenResponseResult {
    pub channel: String,
    pub symbol: String,
    pub event_trigger: KrakenEventTrigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KrakenEventTrigger {
    Bbo,
    Trades,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KrakenConnection {
    pub version: String,
    pub system: String,
    pub api_version: String,
    pub connection_id: u64,
}

impl From<&KrakenTicker> for Ticker {
    fn from(message: &KrakenTicker) -> Self {
        let symbol = TickerSymbol::from_kraken_symbol(&message.symbol);
        Ticker {
            symbol: symbol.unwrap(),
            price: message.last,
            source: Source::Kraken,
            // No timestamp from Kraken :(
            timestamp: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_kraken_subscribe_request() {
        let request = KrakenRequest::Subscribe {
            params: KrakenRequestParams {
                channel: "ticker".to_string(),
                symbol: vec!["ALGO/USD".to_string()],
            },
        };

        let expected_json = json!({
            "method": "subscribe",
            "params": {
                "channel": "ticker",
                "symbol": ["ALGO/USD"]
            }
        });

        let json_value = serde_json::to_value(&request).unwrap();
        assert_eq!(json_value, expected_json);
    }

    #[test]
    fn test_kraken_request_ack() {
        let timestamp = Timestamp::now();
        let expected_json = json!({
            "method": "subscribe",
            "success": true,
            "time_in": timestamp.to_string(),
            "time_out": timestamp.to_string(),
            "result": {
                "channel": "ticker",
                "symbol": "ALGO/USD",
                "snapshot": true,
                "event_trigger": "bbo"
            }
        });

        let response: KrakenResponse = serde_json::from_value(expected_json).unwrap();
        assert!(response.success);
        assert_eq!(response.time_in, timestamp);
        assert_eq!(response.time_out, timestamp);
        match response.response_data {
            KrakenResponseData::Subscribe { result } => {
                assert_eq!(result.channel, "ticker");
                assert_eq!(result.symbol, "ALGO/USD");
                assert!(matches!(result.event_trigger, KrakenEventTrigger::Bbo));
            }
            _ => panic!("Expected Subscribe response"),
        }
    }

    #[test]
    fn test_kraken_ticker_channel_message() {
        let json_value = json!({
                "channel": "ticker",
                "type": "update",
                "data": [
                    {
                        "symbol": "ALGO/USD",
                        "bid": 0.10025,
                        "bid_qty": 740.0,
                        "ask": 0.10035,
                        "ask_qty": 740.0,
                        "last": 0.10035,
                        "volume": 997038.98383185,
                        "vwap": 0.10148,
                        "low": 0.09979,
                        "high": 0.10285,
                        "change": -0.00017,
                        "change_pct": -0.17
                    }
                ]
            }
        );

        let message: KrakenChannelMessage = serde_json::from_value(json_value.clone()).unwrap();
        assert!(matches!(message.data_type, KrakenDataType::Update));
        match message.data {
            KrakenChannelData::Ticker(tickers) => {
                assert_eq!(tickers.len(), 1);
                let ticker = &tickers[0];
                assert_eq!(ticker.bid, dec!(0.10025));
                assert_eq!(ticker.ask, dec!(0.10035));
                assert_eq!(ticker.bid_qty, dec!(740.0));
                assert_eq!(ticker.ask_qty, dec!(740.0));
                assert_eq!(ticker.last, dec!(0.10035));
                assert_eq!(ticker.volume, dec!(997038.98383185));
                assert_eq!(ticker.vwap, dec!(0.10148));
                assert_eq!(ticker.low, dec!(0.09979));
                assert_eq!(ticker.high, dec!(0.10285));
                assert_eq!(ticker.change, dec!(-0.00017));
                assert_eq!(ticker.change_pct, dec!(-0.17));
            }
            _ => panic!("Expected Ticker"),
        }

        let message: KrakenMessage = serde_json::from_value(json_value).unwrap();
        match message {
            KrakenMessage::ChannelMessage(message) => {
                assert!(matches!(message.channel, KrakenChannelType::Ticker));
            }
            _ => panic!("Expected ChannelMessage"),
        }
    }

    #[test]
    fn test_kraken_heartbeat_channel_message() {
        let json_value = json!({
            "channel": "heartbeat",
        });

        let message: KrakenHeartbeat = serde_json::from_value(json_value).unwrap();
        assert!(matches!(message.channel, KrakenChannelType::Heartbeat));
    }

    #[test]
    fn test_kraken_connection_message() {
        let json_value = json!({
            "channel":"status",
            "type":"update",
            "data":[
                {
                    "version":"2.0.9",
                    "system":"online",
                    "api_version":"v2",
                    "connection_id":13221451392339412989_u64
                }
            ]
        });

        let message: KrakenChannelMessage = serde_json::from_value(json_value).unwrap();
        match message.data {
            KrakenChannelData::Connection(connections) => {
                assert_eq!(connections.len(), 1);
                let connection = &connections[0];
                assert_eq!(connection.version, "2.0.9");
                assert_eq!(connection.system, "online");
                assert_eq!(connection.api_version, "v2");
                assert_eq!(connection.connection_id, 13221451392339412989_u64);
            }
            _ => panic!("Expected Connection"),
        }
    }

    #[test]
    fn test_kraken_subscribe_response() {
        let json = r#"{
            "method": "subscribe",
            "result": {
                "channel": "ticker",
                "event_trigger": "trades",
                "snapshot": true,
                "symbol": "BTC/USD"
            },
            "success": true,
            "time_in": "2025-02-14T21:33:53.961562Z",
            "time_out": "2025-02-14T21:33:53.961612Z"
        }"#;

        let response: KrakenResponse = serde_json::from_str(json).unwrap();

        assert!(response.success);
        match response.response_data {
            KrakenResponseData::Subscribe { result } => {
                assert_eq!(result.channel, "ticker");
                assert_eq!(result.symbol, "BTC/USD");
                assert!(result.snapshot.unwrap());
                assert!(matches!(result.event_trigger, KrakenEventTrigger::Trades));
            }
            _ => panic!("Expected Subscribe response"),
        }
    }

    #[test]
    fn test_kraken_subscribe_response_no_snapshot() {
        let json = r#"{
            "method": "unsubscribe",
            "result": {
                "channel": "ticker",
                "event_trigger": "trades",
                "symbol": "BTC/USD"
            },
            "success": true,
            "time_in": "2025-02-14T21:33:53.961562Z",
            "time_out": "2025-02-14T21:33:53.961612Z"
        }"#;

        let response: KrakenResponse = serde_json::from_str(json).unwrap();

        assert!(response.success);
        match response.response_data {
            KrakenResponseData::Unsubscribe { result } => {
                assert_eq!(result.channel, "ticker");
                assert_eq!(result.symbol, "BTC/USD");
                assert!(result.snapshot.is_none());
            }
            _ => panic!("Expected Subscribe response"),
        }
    }
}
