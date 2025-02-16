use std::collections::{HashMap, VecDeque};

use common::{AppInternalMessage, SharedRwRef, Source, Ticker, TickerSymbol};
use feed_processing::FeedProcessor;
use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct SmoothingProcessor {
    config: SharedRwRef<SmoothingConfig>,
    values: SharedRwRef<HashMap<TickerSymbol, VecDeque<Decimal>>>,
    last_emas: SharedRwRef<HashMap<TickerSymbol, Option<Decimal>>>,
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
    pub smoothing: Decimal,
    #[serde(skip)]
    pub smoothing_factor: Decimal,
}

impl SmoothingProcessor {
    pub fn new(config: SharedRwRef<SmoothingConfig>) -> Self {
        let values = SharedRwRef::new(HashMap::new());
        let last_emas = SharedRwRef::new(HashMap::new());
        Self {
            config,
            values,
            last_emas,
        }
    }

    fn calculate_sma(&self, values: &VecDeque<Decimal>, window: u32) -> Option<Decimal> {
        let window = window as usize;
        let sum: Decimal = values.iter().take(window).sum();
        if values.len() >= window {
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
        let config = self.config.read();

        match &*config {
            SmoothingConfig::PassThru => Some(AppInternalMessage::Tickers(tickers.clone())),
            SmoothingConfig::SimpleMovingAverage { params } => {
                let mut values = self.values.write();
                let mut tickers_to_send = Vec::new();

                for ticker in tickers {
                    // Get or create price window for this symbol
                    values
                        .entry(ticker.symbol.clone())
                        .or_default()
                        .push_back(ticker.price);

                    let tickers_for_symbol = values.get_mut(&ticker.symbol).unwrap();

                    // Maintain window size
                    while tickers_for_symbol.len() > params.window as usize {
                        tickers_for_symbol.pop_front();
                    }

                    // Only emit if we have a full window
                    if tickers_for_symbol.len() == params.window as usize {
                        if let Some(smoothed_price) =
                            self.calculate_sma(tickers_for_symbol, params.window)
                        {
                            tickers_to_send.push(Ticker {
                                symbol: ticker.symbol.clone(),
                                price: smoothed_price,
                                source: Source::IndexerSmoothing, // Use consistent source
                                timestamp: Timestamp::now(),
                            });
                        }
                    } else {
                        log::debug!(
                            "Waiting for more prices for {}. Current: {}, Required: {}",
                            ticker.symbol,
                            tickers_for_symbol.len(),
                            params.window
                        );
                    }
                }

                if !tickers_to_send.is_empty() {
                    Some(AppInternalMessage::Tickers(tickers_to_send))
                } else {
                    None
                }
            }
            SmoothingConfig::ExponentialMovingAverage { params } => {
                let mut last_emas = self.last_emas.write();
                let mut tickers_to_send = Vec::new();

                for ticker in tickers {
                    // Get or initialize last EMA for this symbol
                    let last_ema = last_emas.entry(ticker.symbol.clone()).or_insert(None);

                    let ema = self.calculate_ema(ticker.price, *last_ema, params.smoothing_factor);

                    tickers_to_send.push(Ticker {
                        symbol: ticker.symbol.clone(),
                        price: ema,
                        source: ticker.source.clone(),
                        timestamp: Timestamp::now(),
                    });

                    // Update last EMA for this symbol
                    *last_ema = Some(ema);
                }

                if !tickers_to_send.is_empty() {
                    Some(AppInternalMessage::Tickers(tickers_to_send))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{Source, TickerSymbol};
    use jiff::Timestamp;
    use rust_decimal_macros::dec;

    use super::*;

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
                "smoothing": 2.0
            }
        });
        let config: SmoothingConfig = serde_json::from_value(json).unwrap();
        match config {
            SmoothingConfig::ExponentialMovingAverage { params } => {
                assert_eq!(params.window, 10);
                assert_eq!(params.smoothing, dec!(2.0));
            }
            _ => panic!("Expected ExponentialMovingAverage"),
        }
    }
}
