use std::sync::PoisonError;

/// 数据查询过程中出现的错误（不包括权限控制）
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[cfg(feature = "mysql")]
    #[error("connect to database: {0}")]
    ConnError(#[from] r2d2::Error),
    #[error("lock poisoned")]
    PoisonError,
    #[error("data not found")]
    NotFound,
    #[error("send request: {0}")]
    SendRequestError(#[from] awc::error::SendRequestError),
    #[error("store: {0}")]
    StoreError(#[from] store::Error),
    #[error("serde json: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[cfg(feature = "mysql")]
    #[error("diesel: {0}")]
    Diesel(#[from] diesel::result::Error),
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
