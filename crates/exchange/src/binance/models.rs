use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceRequest {
    pub method: BinanceRequestMethod,
    pub params: Vec<String>,
    pub id: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BinanceRequestMethod {
    Subscribe,
    Unsubscribe,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceResponse {
    pub id: u64,
    pub result: Option<String>
}


#[derive(Debug, Clone, Deserialize)]
pub struct BinanceTicker {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E", with = "common::timestamp_millis_serializer")]
    pub event_time: jiff::Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price_change: Decimal,
    #[serde(rename = "P")]
    pub price_change_percent: Decimal,
    #[serde(rename = "w")]
    pub weighted_avg_price: Decimal,
    #[serde(rename = "x")]
    pub first_trade_price: Decimal,
    #[serde(rename = "c")]
    pub last_price: Decimal,
    #[serde(rename = "Q")]
    pub last_quantity: Decimal,
    #[serde(rename = "b")]
    pub best_bid_price: Decimal,
    #[serde(rename = "B")]
    pub best_bid_quantity: Decimal,
    #[serde(rename = "a")]
    pub best_ask_price: Decimal,
    #[serde(rename = "A")]
    pub best_ask_quantity: Decimal,
    #[serde(rename = "o")]
    pub open_price: Decimal,
    #[serde(rename = "h")]
    pub high_price: Decimal,
    #[serde(rename = "l")]
    pub low_price: Decimal,
    #[serde(rename = "v")]
    pub total_traded_base_asset_volume: Decimal,
    #[serde(rename = "q")]
    pub total_traded_quote_asset_volume: Decimal,
    #[serde(rename = "O")]
    pub statistics_open_time: u64,
    #[serde(rename = "C")]
    pub statistics_close_time: u64,
    #[serde(rename = "F")]
    pub first_trade_id: u64,
    #[serde(rename = "L")]
    pub last_trade_id: u64,
    #[serde(rename = "n")]
    pub total_trades: u64,
}


#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_binance_request_serialize() {
        let request = BinanceRequest {
            method: BinanceRequestMethod::Subscribe,
            params: vec!["btcusdt@ticker".to_string()],
            id: 1,
        };
        let json = serde_json::to_value(&request).unwrap();
        let expected_json = json!({
            "method": "SUBSCRIBE",
            "params": ["btcusdt@ticker"],
            "id": 1
        });
        assert_eq!(json, expected_json);
    }


    #[test]
    fn test_binance_response_deserialize() {
        let json = json!({
            "id": 1,
            "result": null
        });
        let response: BinanceResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
    }

    #[test]
    fn test_binance_ticker_deserialize() {
        let json = json!({
            "e": "24hrTicker",
            "E": 1672515782136_u64,
            "s": "BNBBTC",
            "p": "0.0015",
            "P": "250.00",
            "w": "0.0018",
            "x": "0.0009",
            "c": "0.0025",
            "Q": "10",
            "b": "0.0024",
            "B": "10",
            "a": "0.0026",
            "A": "100",
            "o": "0.0010",
            "h": "0.0025",
            "l": "0.0010",
            "v": "10000",
            "q": "18",
            "O": 0,
            "C": 86400000,
            "F": 0,
            "L": 18150,
            "n": 18151
        });
        let ticker: BinanceTicker = serde_json::from_value(json).unwrap();
        assert_eq!(ticker.event_type, "24hrTicker");
        assert_eq!(ticker.event_time, jiff::Timestamp::from_millisecond(1672515782136).unwrap());
        assert_eq!(ticker.symbol, "BNBBTC");
        assert_eq!(ticker.price_change, dec!(0.0015));
        assert_eq!(ticker.price_change_percent, dec!(250.00));
        assert_eq!(ticker.weighted_avg_price, dec!(0.0018));
        assert_eq!(ticker.first_trade_price, dec!(0.0009));
        assert_eq!(ticker.last_price, dec!(0.0025));
        assert_eq!(ticker.last_quantity, dec!(10));
        assert_eq!(ticker.best_bid_price, dec!(0.0024));
        assert_eq!(ticker.best_bid_quantity, dec!(10));
        assert_eq!(ticker.best_ask_price, dec!(0.0026));
        assert_eq!(ticker.best_ask_quantity, dec!(100));
        assert_eq!(ticker.open_price, dec!(0.0010));
        assert_eq!(ticker.high_price, dec!(0.0025));
        assert_eq!(ticker.low_price, dec!(0.0010));
        assert_eq!(ticker.total_traded_base_asset_volume, dec!(10000));
        assert_eq!(ticker.total_traded_quote_asset_volume, dec!(18));
        assert_eq!(ticker.statistics_open_time, 0);
        assert_eq!(ticker.statistics_close_time, 86400000);
        assert_eq!(ticker.first_trade_id, 0);
        assert_eq!(ticker.last_trade_id, 18150);
        assert_eq!(ticker.total_trades, 18151);
    }


}
