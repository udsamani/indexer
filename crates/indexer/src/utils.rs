use common::{AppInternalMessage, Broadcaster, Context, Workers};
use exchange::{BinanceWsClient, CoinbaseWsClient, Exchange, KrakenWsClient};
use feed_processing::FeedProcessingWorker;

use crate::{
    config::{IndexerConfig, IndexerConfigChangeHandler},
    processing::SmoothingProcessor,
};

pub fn add_binance_workers(
    context: &Context,
    workers: &mut Workers,
    app_config: &IndexerConfig,
    broadcaster: Broadcaster<AppInternalMessage>,
    indexer_config_change_handler: &mut IndexerConfigChangeHandler,
) {
    let binance_config = app_config.get_exchange_config(Exchange::Binance);
    let binance_broadcaster = Broadcaster::new(500);
    if let Some(binance_config) = binance_config {
        // Create Feeding Processor for Binance
        let smoothing_processor = SmoothingProcessor::new(
            app_config
                .get_smoothing_config(Exchange::Binance)
                .unwrap()
                .clone(),
        );

        // Create Feeding Processor Worker
        let feeding_processor_worker = FeedProcessingWorker::new(
            context.with_name("binance-feeding-processor-worker"),
            binance_broadcaster.clone(),
            broadcaster,
            smoothing_processor.clone(),
        );

        // Add Feeding Processor Worker to Workers
        workers.add_worker(Box::new(feeding_processor_worker));

        // Create Binance WsClient
        let mut binance_ws_client = BinanceWsClient::new(binance_config.clone());

        // Create Binance WsConsumer
        let binance_consumer =
            binance_ws_client.consumer(context.clone(), binance_broadcaster.sender());

        // Add Binance WsConsumer to IndexerConfigChangeHandler
        indexer_config_change_handler.add_exchange_config_handler(
            Exchange::Binance,
            Box::new(binance_consumer.callback.clone()),
        );
        indexer_config_change_handler
            .add_smoothing_config_handler(Exchange::Binance, Box::new(smoothing_processor));

        // Add Binance WsConsumer to Workers
        workers.add_worker(Box::new(binance_consumer));
    }
}

pub fn add_kraken_workers(
    context: &Context,
    workers: &mut Workers,
    app_config: &IndexerConfig,
    broadcaster: Broadcaster<AppInternalMessage>,
    indexer_config_change_handler: &mut IndexerConfigChangeHandler,
) {
    let kraken_config = app_config.get_exchange_config(Exchange::Kraken);
    let kraken_broadcaster = Broadcaster::new(500);
    if let Some(kraken_config) = kraken_config {
        // Create Feeding Processor for Kraken
        let smoothing_processor = SmoothingProcessor::new(
            app_config
                .get_smoothing_config(Exchange::Kraken)
                .unwrap()
                .clone(),
        );

        // Create Feeding Processor Worker
        let feeding_processor_worker = FeedProcessingWorker::new(
            context.with_name("kraken-feeding-processor-worker"),
            kraken_broadcaster.clone(),
            broadcaster,
            smoothing_processor.clone(),
        );

        // Add Feeding Processor Worker to Workers
        workers.add_worker(Box::new(feeding_processor_worker));

        // Create Kraken WsClient
        let mut kraken_ws_client = KrakenWsClient::new(kraken_config.clone());

        // Create Kraken WsConsumer
        let kraken_consumer =
            kraken_ws_client.consumer(context.clone(), kraken_broadcaster.sender());

        // Add Kraken WsConsumer to IndexerConfigChangeHandler
        indexer_config_change_handler.add_exchange_config_handler(
            Exchange::Kraken,
            Box::new(kraken_consumer.callback.clone()),
        );
        indexer_config_change_handler
            .add_smoothing_config_handler(Exchange::Kraken, Box::new(smoothing_processor));

        // Add Kraken WsConsumer to Workers
        workers.add_worker(Box::new(kraken_consumer));
    }
}

pub fn add_coinbase_workers(
    context: &Context,
    workers: &mut Workers,
    app_config: &IndexerConfig,
    broadcaster: Broadcaster<AppInternalMessage>,
    indexer_config_change_handler: &mut IndexerConfigChangeHandler,
) {
    let coinbase_config = app_config.get_exchange_config(Exchange::Coinbase);
    let coinbase_broadcaster = Broadcaster::new(500);
    if let Some(coinbase_config) = coinbase_config {
        // Create Feeding Processor for Coinbase
        let smoothing_processor = SmoothingProcessor::new(
            app_config
                .get_smoothing_config(Exchange::Coinbase)
                .unwrap()
                .clone(),
        );

        // Create Feeding Processor Worker
        let feeding_processor_worker = FeedProcessingWorker::new(
            context.with_name("coinbase-feeding-processor-worker"),
            coinbase_broadcaster.clone(),
            broadcaster,
            smoothing_processor.clone(),
        );

        // Add Feeding Processor Worker to Workers
        workers.add_worker(Box::new(feeding_processor_worker));

        // Create Coinbase WsClient
        let mut coinbase_ws_client = CoinbaseWsClient::new(coinbase_config.clone());

        // Create Coinbase WsConsumer
        let coinbase_consumer =
            coinbase_ws_client.consumer(context.clone(), coinbase_broadcaster.sender());

        // Add Coinbase WsConsumer to IndexerConfigChangeHandler
        indexer_config_change_handler.add_exchange_config_handler(
            Exchange::Coinbase,
            Box::new(coinbase_consumer.callback.clone()),
        );
        indexer_config_change_handler
            .add_smoothing_config_handler(Exchange::Coinbase, Box::new(smoothing_processor));

        // Add Coinbase WsConsumer to Workers
        workers.add_worker(Box::new(coinbase_consumer));
    }
}
