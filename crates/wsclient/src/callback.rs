use common::AppResult;
use tokio_tungstenite::tungstenite::Message;

#[async_trait::async_trait]
pub trait WsCallback {
    async fn on_connect(&mut self, timestamp: jiff::Timestamp) -> AppResult<()>;
    async fn on_message(&mut self, message: Message, received_time: jiff::Timestamp) -> AppResult<()>;
    fn on_disconnect(&mut self) -> AppResult<()>;
    fn on_heartbeat(&mut self) -> AppResult<()>;
}