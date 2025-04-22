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
}

pub type AppResult<T> = Result<T, AppErrors>;
