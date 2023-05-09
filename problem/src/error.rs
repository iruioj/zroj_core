pub enum Error {
    IO(std::io::Error),
    Zip(zip::result::ZipError),
    SerdeJson(serde_json::Error),
    NoVersion,
    InvalidVersion,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}
impl From<zip::result::ZipError> for Error {
    fn from(value: zip::result::ZipError) -> Self {
        Error::Zip(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
