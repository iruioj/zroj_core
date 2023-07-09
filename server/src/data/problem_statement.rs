use std::io::Read;

use super::error::Error;
use crate::ProblemID;
use async_trait::async_trait;
use problem::render_data::Statement;

pub type StmtDB = dyn Manager + Sync + Send;

#[async_trait]
pub trait Manager {
    /// HTML statement
    async fn get(&self, id: ProblemID) -> Result<String, Error>;
    async fn insert(&mut self, id: ProblemID, reader: impl Read) -> Result<Statement, Error>;
    // fn to_amanager(self) -> std::sync::Arc<StmtDB>;
}
