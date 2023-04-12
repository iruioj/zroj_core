use std::path::PathBuf;

use crate::problem_type::ProblemType;

/// 题目文件
#[derive(Debug)]
pub struct Problem {
    /// 题目文件夹
    pub path: PathBuf, 
    /// 本题数据的配置文件，格式为 yaml
    pub config_path: PathBuf, 
}

impl Problem {
    fn new(path: PathBuf) -> Result<Problem, String> {
        let config_path = path.clone().join("config.yaml");
        Ok(Problem{ path, config_path })
    }

    fn config(&self) -> ProblemType {
        serde_json::from_str(&std::fs::read_to_string(self.config_path.clone()).unwrap()).unwrap()
    }
}

pub fn fuck(p: Problem) {
    dbg!(p);
}

mod tests {
    use super::Problem;
    use super::fuck;

    // #[test]
    // fn test_fuck() {
    //     let p = Problem::new().unwrap();
    //     fuck(p)
    // }
}