
pub enum Error {
    ConnectionError(r2d2::Error),
    DbError(diesel::result::Error),
    LockError
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::ConnectionError(e) => format!("Database connection error: {}", e.to_string()),
            Self::DbError(e) => format!("Database error: {}", e.to_string()),
            Self::LockError => "Lock returned poisoned, which can be caused by a panicked thread".to_string()
        }
    }
}

impl From<Error> for actix_web::Error {
    fn from(value: Error) -> Self {
        actix_web::error::ErrorInternalServerError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;