use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "mysql")]
    ConnectionError(r2d2::Error),
    #[cfg(feature = "mysql")]
    DbError(diesel::result::Error),
    LockError,
    InvalidArgument(String),
    Forbidden(String),
    DuplicatedGroupName(String),
    SerdeJson(serde_json::Error),
    Regex(regex::Error),
    Store(store::Error),
    FetchError(awc::error::SendRequestError),
    IO(std::io::Error),
}

impl From<Error> for actix_web::Error {
    fn from(value: Error) -> Self {
        if let Error::Forbidden(_) = &value {
            return actix_web::error::ErrorForbidden(value.to_string());
        }
        actix_web::error::ErrorInternalServerError(value.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::LockError
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use super::*;
    impl From<r2d2::Error> for Error {
        fn from(value: r2d2::Error) -> Self {
            Self::ConnectionError(value)
        }
    }
    impl From<diesel::result::Error> for Error {
        fn from(value: diesel::result::Error) -> Self {
            Self::DbError(value)
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "mysql")]
            Error::ConnectionError(e) => write!(f, "Database connection error: {}", e),
            #[cfg(feature = "mysql")]
            Error::DbError(e) => write!(f, "Database error: {}", e),
            Error::LockError => write!(
                f,
                "Lock returned poisoned, which can be caused by a panicked thread"
            ),

            Error::InvalidArgument(s) => write!(f, "Invalid argument: {}", s),
            Error::Forbidden(s) => write!(f, "Forbidden operation: {}", s),
            Error::DuplicatedGroupName(s) => write!(f, "duplicated group name {s}"),
            Error::SerdeJson(e) => write!(f, "serialize or deserializing data: {e}"),
            Error::Regex(e) => write!(f, "creating regex: {e}"),
            Error::Store(e) => write!(f, "store error: {e}"),
            Error::FetchError(e) => write!(f, "fetch error: {e}"),
            Error::IO(e) => write!(f, "io: {e}"),
        }
    }
}
impl std::error::Error for Error {}
