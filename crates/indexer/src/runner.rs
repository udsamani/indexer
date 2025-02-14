use common::{static_config, AppResult, Context, Runner, Workers};
use config::Config;
use etcd::EtcdWatcher;

pub struct IndexerRunner {
    context: Context,
}

#[allow(unused)]
impl IndexerRunner {
    pub fn new(context: Context) -> Self {
        Self { context }
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
        log::info!("starting indexer");

        let mut workers = Workers::new(self.context.clone(), 0);

        let etcd_watcher = EtcdWatcher::<String>::from_context(self.context.clone(), "test".to_string());
        workers.add_worker(Box::new(etcd_watcher));

        workers.run().await?;
        Ok("IndexerRunner".to_string())
    }

    fn static_config(&self) -> &Config {
        &self.context.config
    }
}
