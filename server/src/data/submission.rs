use judger::{FileType, Status};

use crate::{SubmID, ProblemID, UserID, manager::problem_judger::FullJudgeReport};

use super::types::*;

pub struct Submission {
    id: SubmID,
    pid: ProblemID,
    uid: UserID,
    submit_time: DateTime,
    judge_time: DateTime,
    /// 评测结果
    report: Option<FullJudgeReport>,
    /// 不是每一个提交记录都有确定的源文件语言
    lang: Option<JsonStr<FileType>>,
    /// 评测状态，None 表示暂无（不一定是评测中）
    status: Option<JsonStr<Status>>,
}