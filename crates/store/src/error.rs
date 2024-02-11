#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("anyerror: {0}")]
    AnyError(#[from] anyhow::Error),
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
