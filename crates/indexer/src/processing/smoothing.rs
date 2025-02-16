use std::collections::VecDeque;

use common::{AppInternalMessage, SharedRwRef, Ticker};
use feed_processing::FeedProcessor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct SmoothingProcessor {
    config: SharedRwRef<SmoothingConfig>,
    values: SharedRwRef<VecDeque<Decimal>>,
    last_ema: SharedRwRef<Option<Decimal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SmoothingConfig {
    #[default]
    PassThru,
    SimpleMovingAverage {
        params: SmaParams,
    },
    ExponentialMovingAverage {
        params: EmaParams,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaParams {
    pub window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaParams {
    pub window: u32,
    pub smoothing_factor: Decimal,
}

impl SmoothingProcessor {
    pub fn new(config: SharedRwRef<SmoothingConfig>) -> Self {
        let values = SharedRwRef::new(VecDeque::new());
        let last_ema = SharedRwRef::new(None);
        Self {
            config,
            values,
            last_ema,
        }
    }

    fn calculate_sma(&self, values: &VecDeque<Decimal>, window: u32) -> Option<Decimal> {
        let window = window as usize;
        if values.len() >= window {
            let sum: Decimal = values.iter().take(window).sum();
            Some(sum / Decimal::from(window))
        } else {
            None
        }
    }

    fn calculate_ema(
        &self,
        current_price: Decimal,
        last_ema: Option<Decimal>,
        smoothing_factor: Decimal,
    ) -> Decimal {
        match last_ema {
            Some(ema) => {
                current_price * smoothing_factor + ema * (Decimal::from(1) - smoothing_factor)
            }
            None => current_price,
        }
    }
}

impl FeedProcessor<AppInternalMessage, AppInternalMessage> for SmoothingProcessor {
    fn process(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        let AppInternalMessage::Tickers(tickers) = input;

        let mut values = self.values.write();
        let config = self.config.read();
        let mut last_ema = self.last_ema.write();

        let processed_tickers = tickers
            .iter()
            .map(|ticker| {
                let smoothed_price = match &*config {
                    SmoothingConfig::PassThru => ticker.price,
                    SmoothingConfig::SimpleMovingAverage { params } => {
                        // Add new price to values
                        values.push_back(ticker.price);

                        // Maintain the window size
                        while values.len() > params.window as usize {
                            values.pop_front();
                        }

                        // Calculate SMA
                        // If we don't have enough values to calculate SMA, return the price, this avoids creating gaps
                        // we should ideally make it configurable
                        self.calculate_sma(&values, params.window)
                            .unwrap_or(ticker.price)
                    }
                    SmoothingConfig::ExponentialMovingAverage { params } => {
                        let ema =
                            self.calculate_ema(ticker.price, *last_ema, params.smoothing_factor);
                        *last_ema = Some(ema);
                        ema
                    }
                };

                Ticker {
                    symbol: ticker.symbol.clone(),
                    price: smoothed_price,
                    source: ticker.source.clone(),
                    timestamp: ticker.timestamp,
                }
            })
            .collect();

        Some(AppInternalMessage::Tickers(processed_tickers))
    }
}

#[cfg(test)]
mod tests {
    use common::{Source, TickerSymbol};
    use jiff::Timestamp;
    use rust_decimal_macros::dec;

    use super::*;

    fn create_ticker(price: Decimal) -> Ticker {
        Ticker {
            symbol: TickerSymbol::BTCUSD,
            price,
            source: Source::Coinbase,
            timestamp: Timestamp::now(),
        }
    }

    fn create_tickers(prices: Vec<Decimal>) -> Vec<Ticker> {
        prices.into_iter().map(create_ticker).collect()
    }

    #[test]
    fn test_sma_basic() {
        let config = SharedRwRef::new(SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 3 },
        });
        let mut processor = SmoothingProcessor::new(config);

        // Test with exactly window size data points
        let input = create_tickers(vec![dec!(10), dec!(20), dec!(30)]);
        let message = AppInternalMessage::Tickers(input);

        if let Some(AppInternalMessage::Tickers(output)) = processor.process(&message) {
            assert_eq!(output[0].price, dec!(10));
            assert_eq!(output[1].price, dec!(20));
            assert_eq!(output[2].price, dec!(20)); // 10 + 20 + 30 / 3
        } else {
            panic!("Expected processed output");
        }
    }

    #[test]
    fn test_sma_insufficient_data() {
        let config = SharedRwRef::new(SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 5 },
        });
        let mut processor = SmoothingProcessor::new(config);

        // Test with less than window size data points
        let input = create_tickers(vec![dec!(10), dec!(20)]);
        let message = AppInternalMessage::Tickers(input);

        if let Some(AppInternalMessage::Tickers(output)) = processor.process(&message) {
            assert_eq!(output[0].price, dec!(10));
            assert_eq!(output[1].price, dec!(20));
        } else {
            panic!("Expected processed output");
        }
    }

    #[test]
    fn test_smoothing_config_deserialization() {
        let json = serde_json::json!({
            "type": "simple_moving_average",
            "params": {
                "window": 10
            }
        });
        let config: SmoothingConfig = serde_json::from_value(json).unwrap();
        match config {
            SmoothingConfig::SimpleMovingAverage { params } => {
                assert_eq!(params.window, 10);
            }
            _ => panic!("Expected SimpleMovingAverage"),
        }
        let json = serde_json::json!({
            "type": "exponential_moving_average",
            "params": {
                "window": 10,
                "smoothing_factor": 0.5
            }
        });
        let config: SmoothingConfig = serde_json::from_value(json).unwrap();
        match config {
            SmoothingConfig::ExponentialMovingAverage { params } => {
                assert_eq!(params.window, 10);
                assert_eq!(params.smoothing_factor, dec!(0.5));
            }
            _ => panic!("Expected ExponentialMovingAverage"),
        }
    }
}
