use std::collections::{HashMap, HashSet};

use common::{AppError, AppInternalMessage, AppResult, SharedRwRef, Source, Ticker, TickerSymbol};
use feed_processing::FeedProcessor;
use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeightedAverageConfig {
    pub weights: HashMap<Source, Decimal>,
    pub stale_threshold_ms: u64,
}

#[allow(unused)]
impl WeightedAverageConfig {
    pub fn validate(&self) -> AppResult<bool> {
        if self.weights.is_empty() {
            return Err(AppError::Unrecoverable(
                "weights cannot be empty".to_string(),
            ));
        }
        let mut total_weight = Decimal::ZERO;
        for weight in self.weights.values() {
            if *weight <= Decimal::ONE {
                return Err(AppError::Unrecoverable(
                    "weight must be greater than 1".to_string(),
                ));
            }
            total_weight += weight;
        }
        if total_weight != Decimal::from(100) {
            return Err(AppError::Unrecoverable(
                "weights must sum to 100".to_string(),
            ));
        }
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct PriceEntry {
    pub price: Decimal,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceKey {
    pub source: Source,
    pub symbol: TickerSymbol,
}

#[allow(unused)]
#[derive(Clone)]
pub struct WeightedAverageProcessor {
    inner: SharedRwRef<InnerWeightedAverageProcessor>,
}

#[allow(unused)]
impl WeightedAverageProcessor {
    pub fn new(config: WeightedAverageConfig) -> AppResult<Self> {
        config.validate()?;
        let inner = InnerWeightedAverageProcessor::new(config);
        Ok(Self {
            inner: SharedRwRef::new(inner),
        })
    }

    fn process_tickers(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        let AppInternalMessage::Tickers(tickers) = input;
        self.inner.write().process_tickers(tickers)
    }
}

struct InnerWeightedAverageProcessor {
    config: WeightedAverageConfig,
    latest_prices: HashMap<PriceKey, PriceEntry>,
}

impl InnerWeightedAverageProcessor {
    pub fn new(config: WeightedAverageConfig) -> Self {
        Self {
            config,
            latest_prices: HashMap::new(),
        }
    }

    fn calculate_weighted_average(
        &self,
        symbol: &TickerSymbol,
        timestamp: Timestamp,
    ) -> Option<Decimal> {
        let mut weighted_sum = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        for (source, weight) in &self.config.weights {
            if let Some(price) = self.latest_prices.get(&PriceKey {
                source: source.clone(),
                symbol: symbol.clone(),
            }) {
                let age = timestamp.duration_since(price.timestamp);
                if age.as_millis() < self.config.stale_threshold_ms as i128 {
                    weighted_sum += price.price * weight;
                    total_weight += weight;
                } else {
                    log::warn!(
                        "Price for {} from {} is too old: {}ms",
                        symbol,
                        source,
                        age.as_millis()
                    );
                }
            }
        }

        // If the total weight is less than 50, don't return a weighted average
        if total_weight > Decimal::from(50) {
            Some(weighted_sum / total_weight)
        } else {
            None
        }
    }

    fn process_tickers(&mut self, tickers: &[Ticker]) -> Option<AppInternalMessage> {
        let now = Timestamp::now();
        let mut weighted_tickers = Vec::new();

        // Group tickers by symbol
        let mut symbols = HashSet::new();

        // Update latest prices
        for ticker in tickers {
            self.latest_prices.insert(
                PriceKey {
                    source: ticker.source.clone(),
                    symbol: ticker.symbol.clone(),
                },
                PriceEntry {
                    price: ticker.price,
                    timestamp: ticker.timestamp,
                },
            );

            // Checking if the symbol is already in the set
            // to avoid cloning the symbol unnecessarily
            if !symbols.contains(&ticker.symbol) {
                symbols.insert(ticker.symbol.clone());
            }
        }

        // Calculate weighted average for each symbol
        for symbol in symbols {
            if let Some(weighted_average) = self.calculate_weighted_average(&symbol, now) {
                weighted_tickers.push(Ticker {
                    symbol: symbol.clone(),
                    price: weighted_average,
                    source: Source::IndexerWeightedAverage,
                    timestamp: now,
                });
            }
        }

        if weighted_tickers.is_empty() {
            None
        } else {
            Some(AppInternalMessage::Tickers(weighted_tickers))
        }
    }
}

impl FeedProcessor<AppInternalMessage, AppInternalMessage> for WeightedAverageProcessor {
    fn process(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        self.process_tickers(input)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_ticker(
        source: Source,
        symbol: TickerSymbol,
        price: Decimal,
        age_ms: u64,
    ) -> Ticker {
        Ticker {
            symbol,
            price,
            source,
            timestamp: Timestamp::now() - Duration::from_millis(age_ms),
        }
    }

    fn setup_processor() -> WeightedAverageProcessor {
        let config = WeightedAverageConfig {
            weights: HashMap::from([
                (Source::Binance, dec!(40)),
                (Source::Kraken, dec!(30)),
                (Source::Coinbase, dec!(30)),
            ]),
            stale_threshold_ms: 5000, // 5 seconds
        };
        WeightedAverageProcessor::new(config).unwrap()
    }

    #[test]
    fn test_basic_weighted_average() {
        let mut processor = setup_processor();

        let tickers = vec![
            create_test_ticker(Source::Binance, TickerSymbol::BTCUSD, dec!(10000), 0),
            create_test_ticker(Source::Kraken, TickerSymbol::BTCUSD, dec!(10100), 0),
            create_test_ticker(Source::Coinbase, TickerSymbol::BTCUSD, dec!(10200), 0),
        ];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(tickers))
        {
            assert_eq!(output[0].price, dec!(10090)); // (10000*0.4 + 10100*0.3 + 10200*0.3)
            assert_eq!(output[0].source, Source::IndexerWeightedAverage);
        } else {
            panic!("Expected weighted average output");
        }
    }

    #[test]
    fn test_stale_prices() {
        let mut processor = setup_processor();

        let tickers = vec![
            create_test_ticker(Source::Binance, TickerSymbol::BTCUSD, dec!(10000), 0), // fresh
            create_test_ticker(Source::Kraken, TickerSymbol::BTCUSD, dec!(10100), 6000), // stale
            create_test_ticker(Source::Coinbase, TickerSymbol::BTCUSD, dec!(10200), 6000), // stale
        ];

        let result = processor.process(&AppInternalMessage::Tickers(tickers));
        assert!(result.is_none()); // Should not produce output with only 40% weight
    }

    #[test]
    fn test_partial_updates() {
        let mut processor = setup_processor();

        // First update - Binance only (40% weight)
        let tickers = vec![create_test_ticker(
            Source::Binance,
            TickerSymbol::BTCUSD,
            dec!(10000),
            0,
        )];
        let result = processor.process(&AppInternalMessage::Tickers(tickers));
        assert!(result.is_none()); // Not enough weight

        // Second update - Kraken (now 70% total weight)
        let tickers = vec![create_test_ticker(
            Source::Kraken,
            TickerSymbol::BTCUSD,
            dec!(10100),
            0,
        )];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(tickers))
        {
            // Expected: (10000*0.4 + 10100*0.3) / (0.4 + 0.3) * 100
            assert_eq!(output[0].price, dec!(10042.857142857142857142857143));
        } else {
            panic!("Expected weighted average with 70% weight coverage");
        }
    }

    #[test]
    fn test_multiple_symbols() {
        let mut processor = setup_processor();

        let tickers = vec![
            create_test_ticker(Source::Binance, TickerSymbol::BTCUSD, dec!(10000), 0),
            create_test_ticker(Source::Kraken, TickerSymbol::BTCUSD, dec!(10100), 0),
            create_test_ticker(Source::Binance, TickerSymbol::ETHUSD, dec!(1000), 0),
            create_test_ticker(Source::Kraken, TickerSymbol::ETHUSD, dec!(1010), 0),
        ];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(tickers))
        {
            assert_eq!(output.len(), 2); // Should have both BTC and ETH

            let btc = output
                .iter()
                .find(|t| t.symbol == TickerSymbol::BTCUSD)
                .unwrap();
            let eth = output
                .iter()
                .find(|t| t.symbol == TickerSymbol::ETHUSD)
                .unwrap();

            // BTC: (10000*0.4 + 10100*0.3) / 0.7 * 100
            assert_eq!(btc.price, dec!(10042.857142857142857142857143));

            // ETH: (1000*0.4 + 1010*0.3) / 0.7 * 100
            assert_eq!(eth.price, dec!(1004.2857142857142857142857143));
        } else {
            panic!("Expected weighted averages for both symbols");
        }
    }

    #[test]
    fn test_price_updates() {
        let mut processor = setup_processor();

        // Initial prices
        let tickers = vec![
            create_test_ticker(Source::Binance, TickerSymbol::BTCUSD, dec!(10000), 0),
            create_test_ticker(Source::Kraken, TickerSymbol::BTCUSD, dec!(10100), 0),
        ];
        processor.process(&AppInternalMessage::Tickers(tickers));

        // Update Binance price
        let tickers = vec![create_test_ticker(
            Source::Binance,
            TickerSymbol::BTCUSD,
            dec!(10500),
            0,
        )];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(tickers))
        {
            // Expected: (10500*0.4 + 10100*0.3) / 0.7 * 100
            assert_eq!(output[0].price, dec!(10328.571428571428571428571429));
        } else {
            panic!("Expected updated weighted average");
        }
    }

    #[test]
    fn test_invalid_config() {
        let config = WeightedAverageConfig {
            weights: HashMap::from([
                (Source::Binance, dec!(40)),
                (Source::Kraken, dec!(40)), // Total 110%
                (Source::Coinbase, dec!(30)),
            ]),
            stale_threshold_ms: 5000,
        };

        assert!(WeightedAverageProcessor::new(config).is_err());
    }
}
