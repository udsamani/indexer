use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Warning(String),
    #[error("{0}")]
    NotImplemented(String),
    #[error("{0}")]
    Unrecoverable(String),
}
