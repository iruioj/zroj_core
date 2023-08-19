#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("open file: {0}")]
    OpenFile(std::io::Error),
    #[error("remove files: {0}")]
    RemoveAll(std::io::Error),
    #[error("not a file")]
    NotFile,
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("creating new file: {0}")]
    CreateNewFile(std::io::Error),
    #[error("creating parent dir: {0}")]
    CreateParentDir(std::io::Error),
    #[error("vector is too long to store")]
    VecTooLong,
}