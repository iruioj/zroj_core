use std::sync::PoisonError;

/// 数据查询过程中出现的错误（不包括权限控制）
///
/// diesel 的 NotFound 会转换为 DataError::NotFound，进而转换为 404
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("connect to database: {0}")]
    ConnError(#[from] r2d2::Error),
    #[error("lock poisoned")]
    PoisonError,
    #[error("data not found")]
    NotFound,
    #[error("try to insert new but already exists")]
    Conflict,
    #[error("send request: {0}")]
    SendRequestError(String),
    #[error("store: {0}")]
    StoreError(#[from] store::Error),
    #[error("serde json: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("diesel: {0}")]
    Diesel(diesel::result::Error),
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
    #[error("from utf8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<diesel::result::Error> for DataError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::NotFound,
            e => Self::Diesel(e),
        }
    }
}

impl From<DataError> for actix_web::Error {
    fn from(value: DataError) -> Self {
        let err_fn = match value {
            DataError::NotFound => actix_web::error::ErrorNotFound,
            _ => actix_web::error::ErrorInternalServerError,
        };
        err_fn(value.to_string())
    }
}

impl<T> From<PoisonError<T>> for DataError {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonError
    }
}
