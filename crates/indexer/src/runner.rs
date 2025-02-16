use crate::{
    config::{IndexerConfig, IndexerConfigChangeHandler},
    processing::WeightedAverageProcessor,
    utils::{add_binance_workers, add_coinbase_workers, add_kraken_workers},
};
use common::{static_config, AppResult, Broadcaster, Context, Runner, Workers};
use config::Config;
use etcd::{EtcdClient, EtcdWatcher};
use feed_processing::FeedProcessingWorker;

pub struct IndexerRunner {
    context: Context,
}

impl IndexerRunner {
    pub async fn get_app_config(
        &self,
        config_key: &str,
        etcd_client: &mut EtcdClient,
    ) -> AppResult<IndexerConfig> {
        let config = etcd_client.get::<IndexerConfig>(config_key).await?;
        Ok(config)
    }
}

impl Default for IndexerRunner {
    fn default() -> Self {
        let config = static_config::create_config(".env/indexer.env")
            .build()
            .unwrap();
        let context = Context::from_config(config);
        Self { context }
    }
}

#[async_trait::async_trait]
impl Runner for IndexerRunner {
    async fn run(&mut self) -> AppResult<String> {
        let config_key = self.context.config.get_string("app_config_key")?;

        let mut workers = Workers::new(self.context.clone(), 0);
        let mut etcd_client = EtcdClient::from_context(&self.context).unwrap();

        // Get App Config Intiailly.
        // We expecte the app config to be present in etcd for initial startup.
        let app_config = self.get_app_config(&config_key, &mut etcd_client).await?;
        log::info!("initial app config: {:?}", app_config);

        let mut indexer_config_change_handler = IndexerConfigChangeHandler::new();
        let broadcaster = Broadcaster::new(2000);

        // Add Binance Workers
        add_binance_workers(
            &self.context,
            &mut workers,
            &app_config,
            broadcaster.clone(),
            &mut indexer_config_change_handler,
        );

        // Add Kraken Workers
        add_kraken_workers(
            &self.context,
            &mut workers,
            &app_config,
            broadcaster.clone(),
            &mut indexer_config_change_handler,
        );

        // Add Coinbase Workers
        add_coinbase_workers(
            &self.context,
            &mut workers,
            &app_config,
            broadcaster.clone(),
            &mut indexer_config_change_handler,
        );

        // Add Weighted Average Processor
        let weighted_average_broadcaster = Broadcaster::new(2000);
        let weighted_average_processor = WeightedAverageProcessor::new(vec![0.5, 0.5]);
        let weighted_average_worker = FeedProcessingWorker::new(
            self.context.clone().with_name("weighted-average-processor"),
            broadcaster.clone(),
            weighted_average_broadcaster.clone(),
            weighted_average_processor,
        );
        workers.add_worker(Box::new(weighted_average_worker));

        // Add EtcdWatcher
        let mut etcd_watcher = EtcdWatcher::<IndexerConfigChangeHandler, IndexerConfig>::new(
            self.context.clone(),
            etcd_client,
            config_key,
        );
        etcd_watcher.add_handler(indexer_config_change_handler);
        workers.add_worker(Box::new(etcd_watcher));

        // Run Workers
        workers.run().await?;
        Ok("IndexerRunner".to_string())
    }

    fn static_config(&self) -> &Config {
        &self.context.config
    }
}
