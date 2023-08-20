use judger::{FileType, Status};
use problem::{Elapse, Memory};

use crate::{manager::problem_judger::FullJudgeReport, ProblemID, SubmID, UserID};

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
    /// 所有测试点中消耗时间的最大值
    time: Option<JsonStr<Elapse>>,
    /// 所有测试点中占用内存的最大值
    memory: Option<JsonStr<Memory>>,
}
