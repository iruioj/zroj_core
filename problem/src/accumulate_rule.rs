/// 子任务记分规则
pub enum AccumulateRule {
    /// 各测试点独立记分
    Independent,
    /// 取各测试点最低分 
    Minimum
}