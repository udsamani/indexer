use std::{marker::PhantomData, time::Duration};

use common::{Context, SpawnResult, Worker};
use serde::de::DeserializeOwned;

use crate::{EtcdClient, ETCD_WATCHER_MESSAGES};

#[derive(Clone)]
pub struct EtcdWatcher<H, C>
where
    H: EtcdWatcherHandler<C>,
    C: DeserializeOwned + Send + 'static + Clone,
{
    context: Context,
    client: EtcdClient,
    handlers: Vec<H>,
    key: String,
    _marker: PhantomData<C>,
}


impl<H, C> EtcdWatcher<H, C>
where
    H: EtcdWatcherHandler<C> + Clone + Send + 'static,
    C: DeserializeOwned + Send + 'static + Clone,
{
    pub fn new(context: Context, client: EtcdClient, key: String) -> Self {
        Self {
            context,
            client,
            handlers: Vec::new(),
            key,
            _marker: PhantomData,
        }
    }

    pub fn from_context(context: Context, key: String) -> Self {
        let client = EtcdClient::from_context(&context).unwrap();
        Self::new(context, client, key)
    }

    pub fn add_handler(&mut self, handler: H) {
        self.handlers.push(handler);
    }
}


impl<H, C> Worker for EtcdWatcher<H, C>
where
    H: EtcdWatcherHandler<C> + Clone + Send + 'static,
    C: DeserializeOwned + Send + 'static + Clone,
{
    fn spawn(&mut self) -> SpawnResult {
        let mut client = self.client.clone();
        let key = self.key.clone();
        let context = self.context.clone();
        let watcher = self.clone();

        tokio::spawn(async move {
            let heartbeat_duration = context.config.get_int("etcd_heartbeat_interval_millis").unwrap_or(5000) as u64;
            let mut heartbeat_interval = tokio::time::interval(Duration::from_millis(heartbeat_duration));
            let mut num_messages_since_last_heartbeat = 0;

            log::info!("starting {} watcher for key: {}", context.name, key);
            let (_watcher, mut stream) = client.watch(&key).await?;

            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        log::info!("{} consumed {} messages since last heartbeat for key: {}", context.name, num_messages_since_last_heartbeat, key);
                        if num_messages_since_last_heartbeat > 0 {
                            ETCD_WATCHER_MESSAGES.with_label_values(&[&key]).add(num_messages_since_last_heartbeat as f64);
                            num_messages_since_last_heartbeat = 0;
                        }
                    }
                    message = stream.message() => {
                        num_messages_since_last_heartbeat += 1;
                        if let Ok(Some(response)) = message {
                            for event in response.events() {
                                if let Some(kv) = event.kv() {
                                    //TODO: handle error
                                    let config: C = serde_json::from_slice::<C>(kv.value()).unwrap();
                                    for handler in watcher.handlers.iter() {
                                        handler.handle_config_change(config.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })


    }
}


pub trait EtcdWatcherHandler<C>
where
    C: DeserializeOwned + Send + 'static + Clone,
{
    fn handle_config_change(&self, config: C);
}
