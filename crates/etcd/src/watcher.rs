use std::time::Duration;

use common::{Context, MpSc, SpawnResult, Worker};

use crate::{EtcdClient, ETCD_WATCHER_MESSAGES};

#[derive(Clone)]
pub struct EtcdWatcher<M> {
    context: Context,
    client: EtcdClient,
    sender: MpSc<M>,
    key: String,
}


impl<M> EtcdWatcher<M> {
    pub fn new(context: Context, client: EtcdClient, sender: MpSc<M>, key: String) -> Self {
        Self { context, client, sender, key }
    }

    pub fn from_context(context: Context, key: String) -> Self {
        let client = EtcdClient::from_context(&context).unwrap();
        let sender = MpSc::new(100);
        Self::new(context, client, sender, key)
    }
}


impl<M> Worker for EtcdWatcher<M> {
    fn spawn(&mut self) -> SpawnResult {
        let mut client = self.client.clone();
        let key = self.key.clone();
        let _sender = self.sender.sender();
        let context = self.context.clone();

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
                    event = stream.message() => {
                        num_messages_since_last_heartbeat += 1;
                        log::info!("{:?}", event);
                    }
                }
            }
        })


    }
}
