use std::path::PathBuf;

/// 题目文件
pub struct Problem {
    /// 题目文件夹
    problem_path: PathBuf, 
    /// 本题数据的配置文件，格式为 yaml
    config_path: PathBuf, 
}

impl Problem {
    fn new() -> Result<Problem, String> {
        todo!() 
    }
}