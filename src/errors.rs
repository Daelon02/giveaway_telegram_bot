use bb8_redis::RedisConnectionManager;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum AppErrors {
    #[error(transparent)]
    DotEnvError(#[from] dotenv::Error),
    #[error(transparent)]
    ParseLevelError(#[from] log::ParseLevelError),
    #[error(transparent)]
    SetLoggerError(#[from] log::SetLoggerError),
    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),
    #[error(transparent)]
    InMemStorageError(#[from] teloxide::dispatching::dialogue::InMemStorageError),
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    RedisPoolError(
        #[from]
        bb8_redis::bb8::RunError<
            <RedisConnectionManager as bb8_redis::bb8::ManageConnection>::Error,
        >,
    ),
    #[error("{0}")]
    StringError(String),
    #[error(transparent)]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppErrors>;
