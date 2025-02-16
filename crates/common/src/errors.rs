use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("exit")]
    Exit,
    #[error("{0}")]
    Warning(String),
    #[error("{0}")]
    NotImplemented(String),
    #[error("{0}")]
    Unrecoverable(String),
    #[error("{0}")]
    ConfigError(String),
    #[error("{0}")]
    EtcdClientError(#[from] etcd_client::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ChannelSendError(String),
    #[error("{0}")]
    WebsocketError(String),
    #[error("{0}")]
    GenericError(String),
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
}

impl From<config::ConfigError> for AppError {
    fn from(error: config::ConfigError) -> Self {
        AppError::ConfigError(error.to_string())
    }
}
