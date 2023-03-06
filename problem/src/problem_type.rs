/// 题目类型，分别对应传统题、交互题、提交答案题
pub enum ProblemType {
    Traditional(ProblemConfig<(PathBuf, PathBuf)>), 
    Interactive(ProblemConfig<(PathBuf, PathBuf)>), 
    AnswerOnly(ProblemConfig<PathBuf>)
}