use std::{path::{PathBuf}, fs::File};
use serde_derive::{Serialize, Deserialize};
use zip::ZipArchive;

use crate::{task::Tasks, accumulate_rule::AccumulateRule};

/// 题目配置文件需要包含的信息，T 表示测试数据的配对方式
#[derive(Serialize, Deserialize)]
pub struct ProblemConfig<T: Checkable> {
    /// 测试数据
    pub tasks: Tasks<T>,
    /// checker 文件相对路径
    pub checker: PathBuf, 
    pub validator: Option<PathBuf>, 
    pub hacker: Option<PathBuf>,
    /// 记分规则  
    pub rule: AccumulateRule, 
    pub time_limit: Option<u32>, 
    pub memory_limit: Option<u32>, 
}

pub trait Checkable {
    fn check(&self, zip: &mut ZipArchive<&File>) -> bool;
}
