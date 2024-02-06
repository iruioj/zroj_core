//! 题目的评测数据

use super::{
    error::DataError,
    file_system::{FileSysDb, FileSysTable},
};
use crate::data::file_system::schema::*;
use crate::ProblemID;
use problem::StandardProblem;

pub type OJDataDB = DefaultDB;

pub struct DefaultDB(FileSysDb);
impl DefaultDB {
    pub fn new(filesysdb: &FileSysDb) -> Result<Self, DataError> {
        Ok(Self(filesysdb.clone()))
    }
}
impl DefaultDB {
    /// the data of problem with `id` is stored in `path/{id}`
    pub fn get(&self, id: ProblemID) -> Result<StandardProblem, DataError> {
        self.0.transaction(|ctx| ojdata::conn(ctx).query(&id))
    }
    pub fn insert(&self, id: ProblemID, mut data: StandardProblem) -> Result<(), DataError> {
        self.0
            .transaction(|ctx| ojdata::conn(ctx).replace(&id, &mut data))
    }
}
