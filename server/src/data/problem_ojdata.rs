//! 题目的评测数据

use super::{
    error::DataError,
    file_system::{FileSysDb, FileSysTable},
};
use crate::data::file_system::schema::*;
use crate::ProblemID;
use problem::StandardProblem;

pub type OJDataDB = dyn Manager + Sync + Send;

pub trait Manager {
    fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError>;
    fn insert(&self, id: ProblemID, data: StandardProblem) -> Result<(), DataError>;
    // 获取数据的元信息用于前端显示
    // fn get_meta(&self, id: ProblemID) -> Result<String, DataError>;
}

pub struct DefaultDB(FileSysDb);
impl DefaultDB {
    pub fn new(filesysdb: &FileSysDb) -> Result<Self, DataError> {
        Ok(Self(filesysdb.clone()))
    }
}
impl Manager for DefaultDB {
    /// the data of problem with `id` is stored in `path/{id}`
    fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError> {
        self.0.transaction(|ctx| ojdata::conn(ctx).query(&id))
    }
    fn insert(&self, id: ProblemID, mut data: StandardProblem) -> Result<(), DataError> {
        self.0
            .transaction(|ctx| ojdata::conn(ctx).replace(&id, &mut data))
    }
}
