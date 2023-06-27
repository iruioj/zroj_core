//! manage problem metadata and store testdata
use crate::ProblemID;
use actix_web::{error, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::RwLock};

/// /{base_dir}/{pid}: problem base directory
///     ./meta.json:  the json file that stores problem statement and general config
///     ./data/: the directory that contains data, there should be a `config.yaml` there
#[derive(Debug)]
pub struct ProblemManager {
    lock: RwLock<()>,
    /// base directory of each problem
    base_dir: String,
}
impl ProblemManager {
    pub fn new(base_dir: String) -> Self {
        Self {
            lock: RwLock::new(()),
            base_dir,
        }
    }
    pub fn get_base_dir(&self, pid: ProblemID) -> Result<PathBuf> {
        let mut path = PathBuf::from(&self.base_dir);
        path = path.join(pid.to_string());
        if path.is_dir() {
            std::fs::create_dir(path.clone()).map_err(|e| {
                error::ErrorInternalServerError(format!(
                    "Fail to establish problem directory {}, error: {}",
                    path.display(),
                    e,
                ))
            })?;
        }
        Ok(path)
    }
    pub fn get_metadata(&self, pid: ProblemID) -> Result<Metadata> {
        let guard = self.lock.read();
        let path = self.get_base_dir(pid)?.join("meta.json");
        let result = std::fs::read_to_string(path.clone()).map_err(|e| {
            error::ErrorInternalServerError(format!(
                "Fail to read from {}, error: {}",
                path.display(),
                e
            ))
        })?;
        let result = serde_json::from_str::<Metadata>(&result).map_err(|e| {
            error::ErrorInternalServerError(format!(
                "Error, the metadata in problem folder is broken: {}",
                e
            ))
        })?;
        drop(guard);
        Ok(result)
    }
}
