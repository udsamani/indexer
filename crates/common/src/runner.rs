use crate::AppResult;

#[async_trait::async_trait]
pub trait Runner {
    async fn run(&mut self) -> AppResult<String>;
}

pub fn run_app<R: Runner>(mut runner: R) {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            runner.run().await.unwrap();
        });
}
