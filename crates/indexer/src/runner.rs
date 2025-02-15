use common::{static_config, AppResult, Context, Runner, Workers};
use config::Config;
use etcd::{EtcdClient, EtcdWatcher};
use exchange::{BinanceWsClient, CoinbaseWsClient, Exchange, KrakenWsClient};
use crate::config::{IndexerConfig, IndexerConfigChangeHandler};

pub struct IndexerRunner {
    context: Context,
}

impl IndexerRunner {
    pub async fn get_app_config(&self, config_key: &str, etcd_client: &mut EtcdClient) -> AppResult<IndexerConfig> {
        let config = etcd_client.get::<IndexerConfig>(config_key).await?;
        Ok(config)
    }
}


impl Default for IndexerRunner {
    fn default() -> Self {
        let config = static_config::create_config(".env/indexer.env").build().unwrap();
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


        // Add Binance WsConsumer
        let binance_config = app_config.get_exchange_config(Exchange::Binance);
        if let Some(binance_config) = binance_config {
            let mut binance_ws_client = BinanceWsClient::new(binance_config.clone());
            let binance_consumer = binance_ws_client.consumer(self.context.clone());
            indexer_config_change_handler.add_handler(Exchange::Binance, Box::new(binance_consumer.callback.clone()));
            workers.add_worker(Box::new(binance_consumer));
        }

        // Add Kraken WsConsumer
        let kraken_config = app_config.get_exchange_config(Exchange::Kraken);
        if let Some(kraken_config) = kraken_config {
            let mut kraken_ws_client = KrakenWsClient::new(kraken_config.clone());
            let kraken_consumer = kraken_ws_client.consumer(self.context.clone());
            indexer_config_change_handler.add_handler(Exchange::Kraken, Box::new(kraken_consumer.callback.clone()));
            workers.add_worker(Box::new(kraken_consumer));
        }

        // Add Coinbase WsConsumer
        let coinbase_config = app_config.get_exchange_config(Exchange::Coinbase);
        if let Some(coinbase_config) = coinbase_config {
            let mut coinbase_ws_client = CoinbaseWsClient::new(coinbase_config.clone());
            let coinbase_consumer = coinbase_ws_client.consumer(self.context.clone());
            indexer_config_change_handler.add_handler(Exchange::Coinbase, Box::new(coinbase_consumer.callback.clone()));
            workers.add_worker(Box::new(coinbase_consumer));
        }

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
