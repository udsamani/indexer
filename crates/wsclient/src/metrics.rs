use lazy_static::lazy_static;
use prometheus as prom;

lazy_static! {
    pub static ref WS_CONSUMER_MESSAGES: prom::CounterVec = prom::register_counter_vec!(
        "ws_consumer_messages",
        "WS consumer messages",
        &["consumer"]
    )
    .unwrap();

    pub static ref WS_MESSAGES_NOT_RECEIVED_CONSECUTIVELY: prom::GaugeVec   = prom::register_gauge_vec!(
        "ws_messages_not_received_consecutively",
        "WS messages not received consecutively",
        &["consumer"]
    )
    .unwrap();
}
