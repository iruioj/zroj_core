//! 题目的评测数据

use super::error::DataError;
use crate::ProblemID;
use anyhow::Context;
use problem::StandardProblem;
use std::collections::BTreeSet;
use store::FsStore;

pub type OJDataDB = dyn Manager + Sync + Send;

pub trait Manager {
    fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError>;
    fn insert(&self, id: ProblemID, data: StandardProblem) -> Result<(), DataError>;
    // 获取数据的元信息用于前端显示
    // fn get_meta(&self, id: ProblemID) -> Result<String, DataError>;
}

#[derive(FsStore, Default)]
struct Data {
    data: BTreeSet<ProblemID>,
}

pub struct DefaultDB(store::Handle);
impl DefaultDB {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, DataError> {
        Ok(Self(store::Handle::new(path)))
    }
}
impl Manager for DefaultDB {
    /// the data of problem with `id` is stored in `path/{id}`
    fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError> {
        let ph = self.0.join(id.to_string());
        if ph.path().exists() {
            Ok(FsStore::open(&ph).context("read problem data")?)
        } else {
            Err(DataError::NotFound)
        }
    }
    fn insert(&self, id: ProblemID, mut data: StandardProblem) -> Result<(), DataError> {
        let ph = self.0.join(id.to_string());
        if ph.path().exists() {
            // 删掉以前的数据（危险的操作，可以考虑加入备份的机制）
            tracing::warn!(?ph, "remove old ojdata");
            std::fs::remove_dir_all(ph.path()).unwrap();
        }
        data.save(&ph).context("write problem data")?;

        Ok(())
    }
}
