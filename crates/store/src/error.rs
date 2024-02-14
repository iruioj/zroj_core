#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("store error: {0}")]
    Error(#[from] anyhow::Error),
}
