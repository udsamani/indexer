use common::{AppError, AppResult, Backoff, Context, MpSc, SharedRef};
use tokio_tungstenite::tungstenite::Message;

use crate::{WsCallback, WsConsumer};

#[derive(Clone)]
pub struct WsClient {
    ws_url: String,
    connected: SharedRef<bool>,
    producer: MpSc<Message>,
    heartbeat_millis: u64,
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64) -> Self {
        Self {
            ws_url,
            connected: SharedRef::new(false),
            producer: MpSc::new(100),
            heartbeat_millis,
        }
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.lock()
    }

    pub fn write(&self, message: Message) -> AppResult<()> {
        match self.producer.sender.try_send(message) {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError::ChannelSendError(format!(
                "failed to send message to ws client: {}",
                e
            ))),
        }
    }

    pub fn close(&self) -> AppResult<()> {
        self.write(Message::Close(None))
    }

    pub fn consumer<C>(&mut self, context: Context, callback: C) -> WsConsumer<C>
    where
        C: WsCallback + Clone,
    {
        WsConsumer {
            ws_url: self.ws_url.clone(),
            callback,
            heartbeat_millis: self.heartbeat_millis,
            backoff: Backoff::default(),
            context,
            mpsc: self.producer.clone_with_receiver(),
        }
    }
}
