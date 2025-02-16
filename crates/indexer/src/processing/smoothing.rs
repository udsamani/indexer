use std::collections::{HashMap, VecDeque};

use common::{AppInternalMessage, AppResult, SharedRwRef, Ticker, TickerSymbol};
use exchange::Exchange;
use feed_processing::FeedProcessor;
use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct SmoothingProcessor {
    inner: SharedRwRef<InnerSmoothingProcessor>,
}

impl SmoothingProcessor {
    pub fn new(config: SmoothingConfig) -> Self {
        let inner = InnerSmoothingProcessor::new(config);
        Self {
            inner: SharedRwRef::new(inner),
        }
    }

    fn process_tickers(&mut self, tickers: &[Ticker]) -> Option<AppInternalMessage> {
        self.inner.write().process_tickers(tickers)
    }
}

#[derive(Clone, Default)]
pub struct InnerSmoothingProcessor {
    config: SmoothingConfig,
    values: HashMap<TickerSymbol, VecDeque<Decimal>>,
    last_emas: HashMap<TickerSymbol, Option<Decimal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SmoothingConfig {
    #[default]
    PassThru,
    #[serde(rename = "sma")]
    SimpleMovingAverage { params: SmaParams },
    #[serde(rename = "ema")]
    ExponentialMovingAverage { params: EmaParams },
}

impl std::fmt::Display for SmoothingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmoothingConfig::PassThru => write!(f, "pass_thru"),
            SmoothingConfig::SimpleMovingAverage { params } => {
                write!(f, "sma(window={})", params.window)
            }
            SmoothingConfig::ExponentialMovingAverage { params } => {
                write!(
                    f,
                    "ema(window={}, smoothing={})",
                    params.window, params.smoothing
                )
            }
        }
    }
}

/// A trait for handling changes to the smoothing configuration
pub trait SmoothingConfigChangeHandler {
    fn handle_config_change(
        &mut self,
        exchange: &Exchange,
        config: SmoothingConfig,
    ) -> AppResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmaParams {
    pub window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmaParams {
    pub window: u32,
    pub smoothing: Decimal,
}

#[allow(unused)]
impl EmaParams {
    pub fn new(window: u32, smoothing: Decimal) -> Self {
        Self { window, smoothing }
    }

    pub fn smoothing_factor(&self) -> Decimal {
        self.smoothing
            .checked_div(Decimal::from(self.window + 1))
            .unwrap()
    }
}

impl InnerSmoothingProcessor {
    pub fn new(config: SmoothingConfig) -> Self {
        let values = HashMap::new();
        let last_emas = HashMap::new();
        Self {
            config,
            values,
            last_emas,
        }
    }

    fn calculate_sma(&mut self, symbol: &TickerSymbol, window: u32) -> Option<Decimal> {
        let tickers_for_symbol = self.values.get_mut(symbol).unwrap();

        // Maintain window size
        while tickers_for_symbol.len() > window as usize {
            tickers_for_symbol.pop_front();
        }
        let window = window as usize;
        let sum: Decimal = tickers_for_symbol.iter().take(window).sum();

        if tickers_for_symbol.len() >= window {
            Some(sum / Decimal::from(window))
        } else {
            log::debug!(
                "waiting for more prices for {}. Current: {}, Required: {}",
                symbol,
                tickers_for_symbol.len(),
                window
            );
            None
        }
    }

    fn calculate_ema(
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

    fn process_tickers_sma(
        &mut self,
        tickers: &[Ticker],
        params: &SmaParams,
    ) -> Option<AppInternalMessage> {
        let mut tickers_to_send = Vec::new();

        for ticker in tickers {
            // Get or create price window for this symbol
            self.values
                .entry(ticker.symbol.clone())
                .or_default()
                .push_back(ticker.price);

            if let Some(smoothed_price) = self.calculate_sma(&ticker.symbol, params.window) {
                tickers_to_send.push(Ticker {
                    symbol: ticker.symbol.clone(),
                    price: smoothed_price,
                    source: ticker.source.clone(),
                    timestamp: Timestamp::now(),
                });
            }
        }

        if !tickers_to_send.is_empty() {
            Some(AppInternalMessage::Tickers(tickers_to_send))
        } else {
            None
        }
    }

    fn process_tickers_ema(
        &mut self,
        tickers: &[Ticker],
        params: &EmaParams,
    ) -> Option<AppInternalMessage> {
        let mut tickers_to_send = Vec::new();

        for ticker in tickers {
            // Get or initialize last EMA for this symbol
            let last_ema = self.last_emas.entry(ticker.symbol.clone()).or_insert(None);

            // Clone price for calculation
            let current_price = ticker.price;
            let ema = Self::calculate_ema(current_price, *last_ema, params.smoothing_factor());

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

    fn process_tickers(&mut self, tickers: &[Ticker]) -> Option<AppInternalMessage> {
        let config = self.config.clone();
        match config {
            SmoothingConfig::PassThru => Some(AppInternalMessage::Tickers(tickers.to_vec())),
            SmoothingConfig::SimpleMovingAverage { params } => {
                self.process_tickers_sma(tickers, &params)
            }
            SmoothingConfig::ExponentialMovingAverage { params } => {
                self.process_tickers_ema(tickers, &params)
            }
        }
    }
}

impl FeedProcessor<AppInternalMessage, AppInternalMessage> for SmoothingProcessor {
    fn process(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        let AppInternalMessage::Tickers(tickers) = input;
        self.process_tickers(tickers)
    }
}

impl SmoothingConfigChangeHandler for SmoothingProcessor {
    fn handle_config_change(
        &mut self,
        exchange: &Exchange,
        config: SmoothingConfig,
    ) -> AppResult<()> {
        let mut inner = self.inner.write();
        if config == inner.config {
            return Ok(());
        }
        log::info!(
            "old config: {} new config: {} for exchange: {}",
            inner.config,
            config,
            exchange
        );
        match (&config, &inner.config) {
            (SmoothingConfig::PassThru, SmoothingConfig::PassThru) => {}
            (
                SmoothingConfig::SimpleMovingAverage { params: _ },
                SmoothingConfig::SimpleMovingAverage { params: _ },
            ) => {
                inner.config = config;
                inner.last_emas.clear();
            }
            (
                SmoothingConfig::ExponentialMovingAverage { params: _ },
                SmoothingConfig::ExponentialMovingAverage { params: _ },
            ) => {
                inner.config = config;
                inner.values.clear();
            }
            _ => {
                inner.config = config;
                inner.values.clear();
                inner.last_emas.clear();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::Source;
    use rust_decimal_macros::dec;

    use super::*;

    fn create_test_ticker(symbol: TickerSymbol, price: Decimal) -> Ticker {
        Ticker {
            symbol,
            price,
            source: Source::Binance,
            timestamp: Timestamp::now(),
        }
    }

    #[test]
    fn test_smoothing_config_deserialization() {
        let json = serde_json::json!({
            "type": "sma",
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
            "type": "ema",
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

    #[test]
    fn test_ema_first_value() {
        let config = SmoothingConfig::ExponentialMovingAverage {
            params: EmaParams::new(10, dec!(2)),
        };
        let mut processor = SmoothingProcessor::new(config);

        // First value should be returned as-is
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(100))];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output[0].price, dec!(100));
        } else {
            panic!("Expected first EMA value");
        }
    }

    #[test]
    fn test_ema_sequence() {
        let config = SmoothingConfig::ExponentialMovingAverage {
            params: EmaParams::new(10, dec!(2)),
        };
        let mut processor = SmoothingProcessor::new(config);

        // Test sequence: 100, 200, 150
        // Alpha = 2/(10+1) ≈ 0.1818
        let alpha = dec!(2) / dec!(11); // ≈ 0.1818

        // First value: 100
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(100))];
        processor.process(&AppInternalMessage::Tickers(input));

        // Second value: 200
        // EMA = 200 * 0.1818 + 100 * 0.8182 = 118.18
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(200))];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output[0].price, dec!(100) + (dec!(200) - dec!(100)) * alpha);
        }

        // Third value: 150
        // EMA = 150 * 0.1818 + 118.18 * 0.8182 = 124.22
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(150))];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            let prev_ema = dec!(100) + (dec!(200) - dec!(100)) * alpha;
            let expected = prev_ema + (dec!(150) - prev_ema) * alpha;
            assert_eq!(output[0].price, expected);
        }
    }

    #[test]
    fn test_multiple_symbols() {
        let config = SmoothingConfig::ExponentialMovingAverage {
            params: EmaParams::new(10, dec!(2)),
        };
        let mut processor = SmoothingProcessor::new(config);

        // Test two symbols simultaneously
        let input = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(100)),
            create_test_ticker(TickerSymbol::ETHUSD, dec!(1000)),
        ];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output.len(), 2);
            assert_eq!(output[0].price, dec!(100));
            assert_eq!(output[1].price, dec!(1000));
        }

        // Update both symbols
        let input = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(200)),
            create_test_ticker(TickerSymbol::ETHUSD, dec!(2000)),
        ];

        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output.len(), 2);
            let alpha = dec!(2) / dec!(11);

            // Check BTC EMA
            let expected_btc = dec!(100) + (dec!(200) - dec!(100)) * alpha;
            assert_eq!(
                output
                    .iter()
                    .find(|t| t.symbol == TickerSymbol::BTCUSD)
                    .unwrap()
                    .price,
                expected_btc
            );

            // Check ETH EMA
            let expected_eth = dec!(1000) + (dec!(2000) - dec!(1000)) * alpha;
            assert_eq!(
                output
                    .iter()
                    .find(|t| t.symbol == TickerSymbol::ETHUSD)
                    .unwrap()
                    .price,
                expected_eth
            );
        }
    }

    #[test]
    fn test_ema_smoothing_factors() {
        // Test different smoothing factors
        let test_cases = vec![
            (dec!(2), dec!(10)), // Traditional EMA
            (dec!(1), dec!(10)), // Slower response
            (dec!(3), dec!(10)), // Faster response
        ];

        for (smoothing, price_change) in test_cases {
            let config = SmoothingConfig::ExponentialMovingAverage {
                params: EmaParams::new(10, smoothing),
            };
            let mut processor = SmoothingProcessor::new(config);

            // Initial price
            let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(100))];
            processor.process(&AppInternalMessage::Tickers(input));

            // Price change
            let input = vec![create_test_ticker(
                TickerSymbol::BTCUSD,
                dec!(100) + price_change,
            )];
            if let Some(AppInternalMessage::Tickers(output)) =
                processor.process(&AppInternalMessage::Tickers(input))
            {
                let alpha = smoothing / dec!(11);
                let expected = dec!(100) + price_change * alpha;
                assert_eq!(output[0].price, expected);
            }
        }
    }

    #[test]
    fn test_sma_window_filling() {
        let config = SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 3 },
        };
        let mut processor = SmoothingProcessor::new(config);

        // First value - no output yet
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(100))];
        let result = processor.process(&AppInternalMessage::Tickers(input));
        assert!(result.is_none());

        // Second value - still no output
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(200))];
        let result = processor.process(&AppInternalMessage::Tickers(input));
        assert!(result.is_none());

        // Third value - now we should get output
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(300))];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output[0].price, dec!(200)); // (100 + 200 + 300) / 3
        } else {
            panic!("Expected SMA output with full window");
        }
    }

    #[test]
    fn test_sma_sliding_window() {
        let config = SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 3 },
        };
        let mut processor = SmoothingProcessor::new(config);

        // Fill window
        let inputs = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(100)),
            create_test_ticker(TickerSymbol::BTCUSD, dec!(200)),
            create_test_ticker(TickerSymbol::BTCUSD, dec!(300)),
        ];
        for input in inputs {
            processor.process(&AppInternalMessage::Tickers(vec![input]));
        }

        // Add new value, should slide window
        let input = vec![create_test_ticker(TickerSymbol::BTCUSD, dec!(400))];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output[0].price, dec!(300)); // (200 + 300 + 400) / 3
        }
    }

    #[test]
    fn test_multiple_symbols_sma() {
        let config = SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 2 },
        };
        let mut processor = SmoothingProcessor::new(config);

        // First update
        let input = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(100)),
            create_test_ticker(TickerSymbol::ETHUSD, dec!(1000)),
        ];
        let result = processor.process(&AppInternalMessage::Tickers(input));
        assert!(result.is_none()); // Window not full yet

        // Second update
        let input = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(200)),
            create_test_ticker(TickerSymbol::ETHUSD, dec!(2000)),
        ];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output.len(), 2);

            let btc = output
                .iter()
                .find(|t| t.symbol == TickerSymbol::BTCUSD)
                .unwrap();
            let eth = output
                .iter()
                .find(|t| t.symbol == TickerSymbol::ETHUSD)
                .unwrap();

            assert_eq!(btc.price, dec!(150)); // (100 + 200) / 2
            assert_eq!(eth.price, dec!(1500)); // (1000 + 2000) / 2
        }
    }

    #[test]
    fn test_partial_updates_sma() {
        let config = SmoothingConfig::SimpleMovingAverage {
            params: SmaParams { window: 3 },
        };
        let mut processor = SmoothingProcessor::new(config);

        // Fill BTC window
        let inputs = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(100)),
            create_test_ticker(TickerSymbol::BTCUSD, dec!(200)),
            create_test_ticker(TickerSymbol::BTCUSD, dec!(300)),
        ];
        for input in inputs {
            processor.process(&AppInternalMessage::Tickers(vec![input]));
        }

        // Add ETH prices (partial window)
        let input = vec![
            create_test_ticker(TickerSymbol::BTCUSD, dec!(400)),
            create_test_ticker(TickerSymbol::ETHUSD, dec!(1000)),
        ];
        if let Some(AppInternalMessage::Tickers(output)) =
            processor.process(&AppInternalMessage::Tickers(input))
        {
            assert_eq!(output.len(), 1); // Only BTC has full window
            assert_eq!(output[0].symbol, TickerSymbol::BTCUSD);
            assert_eq!(output[0].price, dec!(300)); // (200 + 300 + 400) / 3
        }
    }
}
