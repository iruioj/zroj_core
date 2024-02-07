#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("error: {0}")]
    AnyError(#[from] anyhow::Error),
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}