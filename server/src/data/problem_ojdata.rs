//! 题目的评测数据

use super::{error::DataError, fs_store::FsStoreDb};
use crate::ProblemID;
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

pub struct DefaultDB(FsStoreDb);
impl DefaultDB {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, DataError> {
        Ok(Self(FsStoreDb::new(path)))
    }
}
impl Manager for DefaultDB {
    fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError> {
        let table = self.0.table::<Data>();
        table.read_transaction(|db| {
            if db.data.contains(&id) {
                let ctx = table.get_handle().join(id.to_string());
                Ok(FsStore::open(&ctx)?)
            } else {
                Err(DataError::NotFound)
            }
        })
    }
    fn insert(&self, id: ProblemID, mut data: StandardProblem) -> Result<(), DataError> {
        let table = self.0.table::<Data>();
        // let db_ctx = ;
        table.write_transaction(|db| {
            let ctx = table.get_handle().join(id.to_string());
            if ctx.path().exists() {
                // 删掉以前的数据（危险的操作，可以考虑加入备份的机制）
                tracing::warn!(?ctx, "remove old ojdata");
                std::fs::remove_dir_all(ctx.path()).unwrap();
            }
            data.save(&ctx)?;
            db.data.insert(id);
            Ok(())
        })
    }
}
