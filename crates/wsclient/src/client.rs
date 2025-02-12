use common::SharedRef;

pub struct WsClient {
    ws_url: String,
    connected: SharedRef<bool>,
    heartbeat_millis: u64,
}

impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self {
            ws_url: self.ws_url.clone(),
            connected: self.connected.clone(),
            heartbeat_millis: self.heartbeat_millis,
        }
    }
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64) -> Self {
        Self {
            ws_url,
            connected: SharedRef::new(false),
            heartbeat_millis,
        }
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.lock()
    }
}
