use common::{static_config, AppResult, Context, MpSc, Runner, Workers};
use config::Config;
use etcd::{EtcdClient, EtcdWatcher};

use crate::config::IndexerConfig;

pub struct IndexerRunner {
    context: Context,
}

#[allow(unused)]
impl IndexerRunner {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

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
        let _app_config = self.get_app_config(&config_key, &mut etcd_client).await?;
        log::info!("initial app config: {:?}", _app_config);

        let sender = MpSc::<String>::new(100);
        let etcd_watcher = EtcdWatcher::<String>::new(self.context.clone(), etcd_client, sender, config_key);
        workers.add_worker(Box::new(etcd_watcher));

        workers.run().await?;
        Ok("IndexerRunner".to_string())
    }

    fn static_config(&self) -> &Config {
        &self.context.config
    }
}
