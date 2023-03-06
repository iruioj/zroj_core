use serde_derive::{Serialize, Deserialize};

/// 子任务记分规则
#[derive(Serialize, Deserialize)]
pub enum AccumulateRule {
    /// 各测试点独立记分
    Independent,
    /// 取各测试点最低分 
    Minimum
}