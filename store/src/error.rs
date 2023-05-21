#[derive(Debug)]
pub enum Error {
    OpenFile(std::io::Error),
    NotFile,
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    CreateNewFile(std::io::Error),
    CreateParentDir(std::io::Error),
    Custom(String, Box<dyn std::error::Error>),
    VecTooLong,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::OpenFile(e) => write!(f, "error opening file: {}", e),
            Error::NotFile => write!(f, "handle not a file"),
            Error::Deserialize(e) => write!(f, "error deserializing: {}", e),
            Error::CreateNewFile(p) => write!(f, "error creating new file: {}", p),
            Error::Serialize(e) => write!(f, "error serializing: {}", e),
            Error::Custom(s, e) => write!(f, "{}: {}", s, e),
            Error::CreateParentDir(e) => write!(f, "error creating parent dir: {}", e),
            Error::VecTooLong => write!(f, "vector too lang to save"),
        }
    }
}
impl std::error::Error for Error {}

impl Error {
    pub fn new(msg: impl AsRef<str>, e: impl std::error::Error + 'static) -> Self {
        Self::Custom(msg.as_ref().to_string(), Box::new(e))
    }
}
