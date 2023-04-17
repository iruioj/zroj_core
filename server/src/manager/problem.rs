#![allow(dead_code)]
use std::sync::RwLock;

use actix_web::{Result, error};
use serde::{Serialize, Deserialize};
use crate::{config::core::CoreConfig, problem::{ProblemID, ProblemAccess, GeneralConfig, StatementSource}, auth::UserID};

/// For page /problem/{pid}, api url /api/problem/{pid}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemViewData {
    general_config: GeneralConfig,
    statement: StatementViewData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementViewData {
    /// given source code and do client side render
    Markdown(StatementSource),
    /// previously rendered tex into html
    LaTex(String),
}

#[derive(Debug)]
pub struct ProblemManager {
    locks: Vec<RwLock<()>>,
    /// base directory of each problem
    base_dir: String,
    /// the json file that store problem statement
    statement: String,
    /// the directory that stores problem data
    data_dir: String,
    pid_maximum: ProblemID,
}
impl ProblemManager {
    pub fn new(config: &CoreConfig) -> Self {
        Self {
            locks: (0..4000).map(|_| RwLock::new(())).collect(),
            base_dir: config.problem_base_dir.clone(),
            statement: config.problem_statement.clone(),
            data_dir: config.problem_data_dir.clone(),
            pid_maximum: 4000,
        }
    }
    fn fetch_file(&self, path: &String) -> Result <String> {
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
        let guard = self.locks[pid as usize]
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
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
