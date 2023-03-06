use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::config::ProblemConfig;


#[derive(Serialize, Deserialize)]
struct TraditionalData (PathBuf, PathBuf);

/// 题目类型，分别对应传统题、交互题、提交答案题
#[derive(Serialize, Deserialize)]
pub enum ProblemType {
    Traditional(ProblemConfig<(PathBuf, PathBuf)>), 
    Interactive(ProblemConfig<(PathBuf, PathBuf)>), 
    AnswerOnly(ProblemConfig<PathBuf>)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_problem_type() {
        // let x = ProblemType::Interactive(ProblemConfig)
    }
}