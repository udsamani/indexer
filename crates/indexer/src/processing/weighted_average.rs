use common::AppInternalMessage;
use feed_processing::FeedProcessor;

#[allow(unused)]
#[derive(Clone)]
pub struct WeightedAverageProcessor {
    weights: Vec<f64>,
}

#[allow(unused)]
impl WeightedAverageProcessor {
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }
}

impl FeedProcessor<AppInternalMessage, AppInternalMessage> for WeightedAverageProcessor {
    fn process(&mut self, input: &AppInternalMessage) -> Option<AppInternalMessage> {
        Some(input.clone())
    }
}
