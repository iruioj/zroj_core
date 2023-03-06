use std::path::{PathBuf};
use serde_derive::{Serialize, Deserialize};

use crate::{task::Tasks, accumulate_rule::AccumulateRule};

/// 题目配置文件需要包含的信息
#[derive(Serialize, Deserialize)]
pub struct ProblemConfig<T> {
    pub tasks: Tasks<T>,
    pub checker_path: PathBuf, 
    pub validator_path: PathBuf, 
    pub hacker_path: PathBuf,
    /// 记分规则  
    pub rule: AccumulateRule, 
    pub time_limit: Option<i32>, 
    pub memory_limit: Option<i32>, 
}
