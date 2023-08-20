use super::types::*;
use crate::{manager::problem_judger::FullJudgeReport, ProblemID, SubmID, UserID};
#[cfg(feature = "mysql")]
use diesel::*;
use judger::FileType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "mysql", derive(Queryable, AsChangeset))]
#[cfg_attr(feature = "mysql", diesel(table_name = database::submissions))]
pub struct Submission {
    id: SubmID,
    pid: ProblemID,
    uid: UserID,
    submit_time: DateTime,
    judge_time: DateTime,
    /// 评测结果
    report: Option<JsonStr<FullJudgeReport>>,
    /// 不是每一个提交记录都有确定的源文件语言
    lang: Option<JsonStr<FileType>>,
    /// 评测状态，None 表示暂无（不一定是评测中）
    status: Option<JsonStr<judger::Status>>,
    /// 所有测试点中消耗时间的最大值
    time: Option<CastElapse>,
    // 所有测试点中占用内存的最大值
    memory: Option<CastMemory>,
}

#[cfg(feature = "mysql")]
mod database {
    use diesel::{self, table};
    table! {
        submissions (id) {
            /// id should be auto increment
            id -> Unsigned<Integer>,
            pid -> Unsigned<Integer>,
            uid -> Unsigned<Integer>,
            submit_time -> BigInt,
            judge_time -> BigInt,
            report -> Text,
            lang -> Text,
            status -> Text,
            time -> Unsigned<BigInt>,
            memory -> Unsigned<BigInt>,
        }
    }
}