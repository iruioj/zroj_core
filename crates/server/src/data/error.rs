//! Define error types used by database operations.
use std::sync::PoisonError;

use crate::UserID;

/// 数据查询过程中出现的错误（不包括权限控制）
///
/// diesel 的 NotFound 会转换为 DataError::NotFound，进而转换为 404
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("data not found")]
    NotFound,
    #[error("data error: {0}")]
    Error(#[from] anyhow::Error),
    #[error("no permission: user_id = {0}, perm_id = {1}")]
    Perm(UserID, u64),
}

impl From<diesel::result::Error> for DataError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::NotFound,
            e => Self::Error(anyhow::anyhow!("diesel error: {e}")),
        }
    }
}

impl From<DataError> for actix_web::Error {
    fn from(value: DataError) -> Self {
        let err_fn = match value {
            DataError::NotFound => actix_web::error::ErrorNotFound,
            _ => actix_web::error::ErrorInternalServerError,
        };
        err_fn(value)
    }
}

impl<T> From<PoisonError<T>> for DataError {
    fn from(e: PoisonError<T>) -> Self {
        Self::Error(anyhow::anyhow!("poison error: {e}"))
    }
}
