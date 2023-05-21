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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    statement: Statement,
    config: GeneralConfig,
}

/// problem statement, stored in self.statement_path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Markdown(StatementSource),
    LaTex(StatementSource),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementSource {
    /// load from an pdf asset
    Pdf(PathBuf),
    /// directly load from html
    Legacy(String),
    /// standard form, consists of several parts(html) and will be rendered in a different format
    Standard {
        /// problem background & description
        legend: String,
        /// input format
        input_format: String,
        /// output format
        output_format: String,
        /// notes & constraints & case/subtask specification
        notes: String,
        /// examples, either user input or generated by running a testcase
        examples: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub id: ProblemID,
    /// Some problem may have different time limit on each testcase
    /// in which case should the Judger Parse the limit and return a {min}~{max}ms
    pub time_limit: String,
    /// Like time limit, but MB not ms
    pub memory_limit: String,
    /// Problem type
    pub problem_type: ProblemType,
}
/// For traditional problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemIOType {
    /// read from stdin, write to stdout
    StandardIO,
    /// specify files to read and write
    /// some.in, some.out
    FileIO(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemType {
    /// Traditional, also supports NOI style interactive problem
    Traditional(ProblemIOType),
    /// I/O Interactive Problem
    Interactive,
    /// Submit answer only
    SubmitAnswer,
}
