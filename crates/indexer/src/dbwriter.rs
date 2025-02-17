use std::time::Duration;

use common::{AppError, AppInternalMessage, AppResult, Broadcaster, Context, SpawnResult, Worker};
use lazy_static::lazy_static;
use prometheus as prom;
use tokio::sync::broadcast::error::RecvError;
use tokio_postgres::Client;

lazy_static! {
    pub static ref DBWRITER_MESSAGES_WRITTEN: prom::CounterVec = prom::register_counter_vec!(
        "dbwriter_messages_written",
        "DB writer messages written",
        &[]
    )
    .unwrap();
    pub static ref DBWRITER_MESSAGES_DROPPED: prom::CounterVec = prom::register_counter_vec!(
        "dbwriter_messages_dropped",
        "DB writer messages dropped",
        &[]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct DbWriter {
    context: Context,
    broadcaster: Broadcaster<AppInternalMessage>,
    database_url: String,
    insertion_interval_ms: u64,
    messages: Vec<AppInternalMessage>,
}

impl DbWriter {
    pub fn new(context: Context, broadcaster: Broadcaster<AppInternalMessage>) -> AppResult<Self> {
        let database_url = context.config.get_string("database_url")?;
        let insertion_interval_ms = context
            .config
            .get_int("database_insertion_interval_ms")
            .unwrap_or(2000) as u64;
        Ok(Self {
            context,
            broadcaster,
            database_url,
            insertion_interval_ms,
            messages: vec![],
        })
    }
}

impl Worker for DbWriter {
    fn spawn(&mut self) -> SpawnResult {
        let mut dbwriter = self.clone();
        tokio::spawn(async move {
            let (mut client, connection) =
                tokio_postgres::connect(&dbwriter.database_url, tokio_postgres::NoTls)
                    .await
                    .map_err(AppError::PostgresError)?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    log::error!("database connection error: {}", e);
                }
            });
            let mut app = dbwriter.context.app.subscribe();
            let mut receiver = dbwriter.broadcaster.receiver();
            let mut insertion_interval =
                tokio::time::interval(Duration::from_millis(dbwriter.insertion_interval_ms));

            loop {
                tokio::select! {
                    _ = app.recv() => {
                        log::info!("{} received exit signal", dbwriter.context.name);
                        return Ok(format!("{} exited", dbwriter.context.name));
                    }
                    message = receiver.recv() => {
                        match message {
                            Ok(message) => {
                                dbwriter.messages.push(message);
                            },
                            Err(RecvError::Lagged(lag)) => {
                                DBWRITER_MESSAGES_DROPPED.with_label_values(&[]).inc_by(lag as f64);
                                log::warn!("{} broadcaster receiver lagged by {}", dbwriter.context.name, lag);
                                continue;
                            },
                            Err(e) => {
                                log::error!("broadcaster receiver error: {}", e);
                            }
                        }
                    }
                    _ = insertion_interval.tick() => {
                        let num_messages = dbwriter.messages.len();
                        if !dbwriter.messages.is_empty() {
                            insert_messages(dbwriter.messages.drain(..).collect(), &mut client).await?;
                            DBWRITER_MESSAGES_WRITTEN.with_label_values(&[]).inc_by(num_messages as f64);
                        }
                    }
                }
            }
        })
    }
}

pub async fn insert_messages(
    messages: Vec<AppInternalMessage>,
    client: &mut Client,
) -> AppResult<()> {
    let mut flat_tickers = Vec::new();
    for message in messages {
        match message {
            AppInternalMessage::Tickers(mut tickers) => {
                flat_tickers.append(&mut tickers);
            }
        }
    }

    let values: Vec<_> = flat_tickers
        .iter()
        .map(|ticker| {
            format!(
                "('{}', {}, {}, '{}')",
                ticker.symbol,
                ticker.price,
                ticker.timestamp.as_millisecond(),
                ticker.source
            )
        })
        .collect();

    let query = format!(
        "INSERT INTO tickers (symbol, price, timestamp, source) VALUES {}",
        values.join(", ")
    );

    client
        .execute(&query, &[])
        .await
        .map_err(AppError::PostgresError)?;
    Ok(())
}
