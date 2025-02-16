use common::{AppError, AppResult, Backoff, Context, MpSc, SpawnResult, Worker};
use futures_util::{SinkExt, StreamExt};
use jiff::Timestamp;
use std::time::Duration;
use tokio::{io, sync::mpsc::Receiver};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::WsCallback;

#[derive(Clone)]
pub struct WsConsumer<C>
where
    C: WsCallback + Clone,
{
    pub ws_url: String,
    pub callback: C,
    pub heartbeat_millis: u64,
    pub backoff: Backoff,
    pub context: Context,
    pub mpsc: MpSc<Message>,
}

impl<C> WsConsumer<C>
where
    C: Clone + WsCallback,
{
    pub async fn run(&mut self) -> AppResult<String> {
        let mut receiver = self.mpsc.receiver().unwrap();

        loop {
            match self.backoff.next() {
                Some(delay_secs) => {
                    if delay_secs > 0 {
                        tokio::time::sleep(Duration::from_secs(delay_secs as u64)).await;
                    }
                }
                None => {
                    return Err(AppError::WebsocketError(format!(
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
                    log::error!("failed to connect to websocket: {}", e);
                    continue;
                }
            };

            let stream_result = self.stream(&mut receiver, ws_stream).await;
            self.on_disconnect()?;

            match stream_result {
                Ok(_) => {
                    log::warn!("websocket {} disconnected", self.context.name);
                    return Ok(format!("websocket {} disconnected", self.context.name));
                }
                Err(AppError::Exit) => {
                    return Err(AppError::Exit);
                }
                Err(AppError::GenericError(e)) => {
                    log::error!("error while streaming websocket: {}", e);
                }
                Err(e) => {
                    log::error!("error while streaming websocket: {}", e);
                }
            }
        }
    }

    async fn stream<S>(
        &mut self,
        receiver: &mut Receiver<Message>,
        mut ws_stream: WebSocketStream<S>,
    ) -> AppResult<()>
    where
        S: io::AsyncRead + io::AsyncWrite + Unpin + Send + 'static,
    {
        self.on_connect().await?;
        let mut app = self.context.app.subscribe();
        let mut num_messages_since_last_heartbeat = 0;
        let mut heartbeat = tokio::time::interval(Duration::from_millis(self.heartbeat_millis));

        loop {
            tokio::select! {
                _ = app.recv() => {
                    if let Err(e) = ws_stream.send(Message::Close(None)).await {
                        return Err(AppError::GenericError(format!("error while sending close message to websocket: {}", e)));
                    }
                    return Ok(());
                }
                result = ws_stream.next() => {
                    match result {
                        Some(result) => {
                            let received_time = Timestamp::now();
                            num_messages_since_last_heartbeat += 1;
                            match result {
                                Ok(message) => {
                                    self.callback.on_message(message, received_time).await?;
                                }
                                Err(e) => {
                                    return Err(AppError::GenericError(format!("error while streaming websocket: {}", e)));
                                }
                            };
                        }
                        None => {
                            return Err(AppError::GenericError("websocket stream closed".to_string()));
                        }
                    }
                }
                result = receiver.recv() => {
                    match result {
                        Some(message) => {
                            if let Err(e) = ws_stream.send(message).await {
                                return Err(AppError::GenericError(format!("error while sending message to websocket: {}", e)));
                            }
                            log::debug!("sent message to websocket: {}", self.ws_url);
                        }
                        None => {
                            return Err(AppError::GenericError("receiver closed".to_string()));
                        }
                    }
                }
                _ = heartbeat.tick() => {
                    let _ = self.callback.on_heartbeat();
                    log::info!("{} received {} messages since last heartbeat", self.context.name, num_messages_since_last_heartbeat);
                    num_messages_since_last_heartbeat = 0;
                }
            }
        }
    }

    async fn on_connect(&mut self) -> AppResult<()> {
        let timestamp = Timestamp::now();
        self.callback.on_connect(timestamp).await
    }

    fn on_disconnect(&mut self) -> AppResult<()> {
        self.callback.on_disconnect()
    }
}

impl<C> Worker for WsConsumer<C>
where
    C: Clone + WsCallback + Send + 'static,
{
    fn spawn(&mut self) -> SpawnResult {
        let mut consumer = self.clone();
        consumer.mpsc = self.mpsc.clone_with_receiver();
        tokio::spawn(async move { consumer.run().await })
    }
}
