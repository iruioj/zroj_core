use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SubTask<T> {
    /// 子任务的显示名称
    pub name: String, 
    /// 这个子任务的测试数据，要求输入文件和（可能的）答案（或者评分）文件配对。提交答案题可能不需要答案文件。
    pub tests: Vec<T>,
    /// 不同子任务可能有不同的时空限制
    pub time_limit: Option<i32>, 
    pub memory_limit: Option<i32>, 
    /// 子任务总分
    pub score: f32
}

#[derive(Serialize, Deserialize)]
pub struct TestCase<T> {
    pub name: String, 
    pub test: T
}

#[derive(Serialize, Deserialize)]
pub enum Tasks<T> {
    Subtasks(Vec<SubTask<T>>, Vec<(usize, usize)>), 
    TestCases(Vec<TestCase<T>>)
}