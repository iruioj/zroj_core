#![allow(dead_code)]
use crate::{
    auth::UserID,
    problem::{GeneralConfig, ProblemAccess, ProblemID, StatementSource},
};
use actix_web::{error, Result};
use serde::Serialize;
use std::sync::RwLock;

/// For page /problem/{pid}, api url /api/problem/{pid}
#[derive(Debug, Clone, Serialize)]
pub struct ProblemViewData {
    general_config: GeneralConfig,
    statement: StatementSource,
}

#[derive(Debug)]
pub struct ProblemManager {
    lock: RwLock<()>,
    /// base directory of each problem
    base_dir: String,
    /// the json file that store problem statement
    statement: String,
    /// the directory that stores problem data
    data_dir: String,
    pid_maximum: ProblemID,
}
impl ProblemManager {
    pub fn new(base_dir: String, statement: String, data_dir: String) -> Self {
        Self {
            lock: RwLock::new(()),
            base_dir,
            statement,
            data_dir,
            pid_maximum: 4000,
        }
    }
    fn fetch_file(&self, path: &String) -> Result<String> {
        std::fs::read_to_string(path).map_err(|e| error::ErrorInternalServerError(e.to_string()))
    }
    fn get_base_dir(&self, pid: ProblemID) -> Result<String> {
        let mut s = self.base_dir.clone();
        if let None = s.find("{}") {
            return Err(error::ErrorInternalServerError(
                "Problem base dir is not correct. {} is required".to_string(),
            ));
        }
        s = s.replace("{}", &pid.to_string());
        if let Some(_) = s.find("{}") {
            return Err(error::ErrorInternalServerError(
                "Problem base dir is not correct. Too many {}s".to_string(),
            ));
        }
        Ok(s)
    }
    fn read_statement(&self, pid: ProblemID) -> Result<String> {
        let guard = self.lock .read() .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let dir = self.get_base_dir(pid)? + &self.statement;
        let result = self.fetch_file(&dir)?;
        drop(guard);
        Ok(result)
    }
    pub fn check_access(&self, _pid: ProblemID, _uid: UserID) -> actix_web::Result<ProblemAccess> {
        todo!()
    }
    pub fn fetch_view_data(&self, _pid: ProblemID) -> actix_web::Result<ProblemViewData> {
        todo!()
    }
}
