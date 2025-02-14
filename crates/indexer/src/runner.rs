use common::{static_config, AppResult, Context, Runner};
use config::Config;

pub struct IndexerRunner {
    _context: Context,
}

#[allow(unused)]
impl IndexerRunner {
    pub fn new(context: Context) -> Self {
        Self { _context: context }
    }
}


impl Default for IndexerRunner {
    fn default() -> Self {
        let config = static_config::create_config(".env/indexer.env").build().unwrap();
        let context = Context::from_config(config);
        Self { _context: context }
    }
}


#[async_trait::async_trait]
impl Runner for IndexerRunner {
    async fn run(&mut self) -> AppResult<String> {
        log::info!("starting indexer");
        tokio::time::sleep(tokio::time::Duration::from_millis(100000)).await;
        Ok("IndexerRunner".to_string())
    }

    fn static_config(&self) -> &Config {
        &self._context.config
    }
}
