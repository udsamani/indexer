use lazy_static::lazy_static;
use prometheus as prom;

lazy_static! {
    pub static ref WS_CONSUMER_MESSAGES: prom::CounterVec = prom::register_counter_vec!(
        "ws_consumer_messages",
        "WS consumer messages",
        &["consumer"]
    )
    .unwrap();
}
