use lazy_static::lazy_static;
use prometheus as prom;

lazy_static! {
    pub static ref ETCD_WATCHER_MESSAGES: prom::GaugeVec = prom::register_gauge_vec!(
        "etcd_watcher_messages",
        "Etcd watcher messages",
        &["key"]
    )
    .unwrap();

    pub static ref ETCD_WATCHER_KEY_UPDATES: prom::CounterVec = prom::register_counter_vec!(
        "etcd_watcher_key_updates",
        "Etcd watcher key updates",
        &["key"]
    )
    .unwrap();

}
