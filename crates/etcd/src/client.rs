use common::{AppError, AppResult, Context, SharedAsyncRef};
use serde::de::DeserializeOwned;

/// A ETCD client that can be shared across threads.
///
/// This is a lazy wrapper around an etcd client that can be shared across threads.
#[derive(Clone)]
pub struct EtcdClient {
    etcd_url: String,
    etcd_conn_manager: SharedAsyncRef<EtcdConnectionManager>,
}

#[derive(Default)]
struct EtcdConnectionManager {
    etcd_conn_manager: Option<etcd_client::Client>,
}

impl EtcdClient {
    /// Create a new ETCD client from a context
    pub fn from_context(context: &Context) -> AppResult<Self> {
        let etcd_url = context.etcd_url()?;
        Ok(Self::new(&etcd_url))
    }

    /// Create a new ETCD client from a etcd url
    pub fn new(etcd_url: &str) -> Self {
        Self {
            etcd_url: etcd_url.to_string(),
            etcd_conn_manager: SharedAsyncRef::default(),
        }
    }

    pub async fn connection_manager(&mut self) -> AppResult<etcd_client::Client> {
        self.etcd_conn_manager
            .lock()
            .await
            .get(&self.etcd_url)
            .await
    }

    /// Watch a key
    pub async fn watch(
        &mut self,
        key: &str,
    ) -> AppResult<(etcd_client::Watcher, etcd_client::WatchStream)> {
        let mut client = self.connection_manager().await?;
        let (watcher, stream) = client.watch(key, None).await?;
        Ok((watcher, stream))
    }

    /// Get a key
    pub async fn get<M: DeserializeOwned>(&mut self, key: &str) -> AppResult<M> {
        let mut client = self.connection_manager().await?;
        let response = client.get(key, None).await?;

        let value = response
            .kvs()
            .first()
            .ok_or_else(|| AppError::ConfigError(format!("key {} not found", key)))
            .and_then(|kv| {
                serde_json::from_slice::<M>(kv.value()).map_err(AppError::SerdeJsonError)
            })?;

        Ok(value)
    }
}

impl EtcdConnectionManager {
    pub async fn get(&mut self, etcd_url: &str) -> AppResult<etcd_client::Client> {
        if self.etcd_conn_manager.is_none() {
            let client = etcd_client::Client::connect(vec![etcd_url], None).await?;
            self.etcd_conn_manager = Some(client);
        }
        Ok(self.etcd_conn_manager.clone().unwrap())
    }
}
