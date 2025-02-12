use std::time::Duration;

use crate::WsCallback;
use common::{AppError, AppResult, Backoff};

pub struct WsConsumer<C>
where
    C: WsCallback,
{
    pub ws_url: String,
    pub callback: C,
    pub backoff: Backoff,
}

impl<C> WsConsumer<C>
where
    C: WsCallback,
{
    pub async fn run(&mut self) -> AppResult<String> {
        loop {
            match self.backoff.next() {
                Some(delay_secs) => {
                    if delay_secs > 0 {
                        tokio::time::sleep(Duration::from_secs(delay_secs as u64)).await;
                    }
                }
                None => {
                    return Err(AppError::Unrecoverable(format!(
                        "failed to connect to websocket after {} attempts",
                        self.backoff.get_iteration_count()
                    )));
                }
            }

            log::info!("connecting to websocket: {}", self.ws_url);
            let ws_stream = match tokio_tungstenite::connect_async(&self.ws_url).await {
                Ok((ws_stream, _)) => {
                    log::info!("connected to websocket: {}", &self.ws_url);
                    self.backoff.reset();
                    ws_stream
                }
                Err(e) => {
                    log::warn!(
                        "failed to connect to websocket: {} with error: {}",
                        &self.ws_url,
                        e
                    );
                    continue;
                }
            };
        }
        Ok("success".to_string())
    }


    
}
