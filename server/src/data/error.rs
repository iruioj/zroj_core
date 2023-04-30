
pub enum Error {
    ConnectionError(r2d2::Error),
    DbError(diesel::result::Error)
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::ConnectionError(e) => format!("Database connection error: {}", e.to_string()),
            Self::DbError(e) => format!("Database error: {}", e.to_string()),
        }
    }
}


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

impl From<Error> for actix_web::Error {
    fn from(value: Error) -> Self {
        actix_web::error::ErrorInternalServerError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;