use common::{AppInternalMessage, Ticker};
use feed_processing::FeedProcessor;

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub struct SmoothingProcessor {
    pub smoothing_type: SmoothingType,
}

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub enum SmoothingType {
    #[default]
    PassThrough,
    Exponential,
    Simple,
}

#[allow(unused)]
impl SmoothingProcessor {
    pub fn new(smoothing_type: SmoothingType) -> Self {
        Self { smoothing_type }
    }
}

impl FeedProcessor<AppInternalMessage, AppInternalMessage> for SmoothingProcessor {
    fn process(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        let AppInternalMessage::Tickers(tickers) = input;

        let tickers = tickers
            .iter()
            .map(|ticker| Ticker {
                symbol: ticker.symbol.clone(),
                price: ticker.price,
                source: common::Source::Indexer,
                timestamp: ticker.timestamp,
            })
            .collect();

        Some(AppInternalMessage::Tickers(tickers))
    }
}
