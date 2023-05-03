#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "mysql")]
    ConnectionError(r2d2::Error),
    DbError(diesel::result::Error),
    LockError,
    InvalidArgument(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            #[cfg(feature = "mysql")]
            Self::ConnectionError(e) => format!("Database connection error: {}", e.to_string()),
            Self::DbError(e) => format!("Database error: {}", e.to_string()),
            Self::LockError => {
                "Lock returned poisoned, which can be caused by a panicked thread".to_string()
            }
            Self::InvalidArgument(s) => format!("Invalid argument: {}", s),
        }
    }
}

impl From<Error> for actix_web::Error {
    fn from(value: Error) -> Self {
        actix_web::error::ErrorInternalServerError(value.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::LockError
    }
}

#[cfg(feature = "mysql")]
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

pub type Result<T> = std::result::Result<T, Error>;
