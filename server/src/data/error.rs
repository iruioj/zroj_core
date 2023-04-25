enum Error {
    DbError,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl From<Error> for actix_web::Error {
    fn from(value: Error) -> Self {
        actix_web::error::ErrorInternalServerError(value.to_string())
    }
}