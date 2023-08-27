use super::{error::DataError, types::*};
use crate::{manager::problem_judger::FullJudgeReport, ProblemID, SubmID, UserID};
use async_trait::async_trait;
#[cfg(feature = "mysql")]
use diesel::*;
use serde::{Deserialize, Serialize};

/// 提交记录的元信息，不包含源代码、评测日志等
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "mysql", derive(Queryable, AsChangeset))]
#[cfg_attr(feature = "mysql", diesel(table_name = database::submissions))]
pub struct Submission {
    pub id: SubmID,
    pid: ProblemID,
    uid: UserID,
    submit_time: DateTime,
    judge_time: Option<DateTime>,
    /// 评测结果
    report: Option<JsonStr<FullJudgeReport>>,
    /// 不是每一个提交记录都有确定的源文件语言
    lang: Option<JsonStr<judger::FileType>>,
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
            judge_time -> Nullable<BigInt>,
            report -> Nullable<Text>,
            lang -> Nullable<Text>,
            status -> Nullable<Text>,
            time -> Nullable<Unsigned<BigInt>>,
            memory -> Nullable<Unsigned<BigInt>>,
        }
    }
}

pub type SubmDB = dyn Manager + Sync + Send;

#[async_trait(?Send)]
pub trait Manager {
    async fn insert_new(&self, uid: UserID, pid: ProblemID) -> Result<Submission, DataError>;
}

mod default {
    use std::{collections::BTreeMap, sync::RwLock};

    use crate::SubmID;

    use super::*;

    struct DefaultDB {
        data: RwLock<BTreeMap<SubmID, Submission>>,
    }

    #[async_trait(?Send)]
    impl super::Manager for DefaultDB {
        async fn insert_new(&self, uid: UserID, pid: ProblemID) -> Result<Submission, DataError> {
            let mut data = self.data.write()?;
            let id = data.iter().next_back().map(|x| *x.0).unwrap_or(0) + 1;
            let r = Submission {
                id,
                pid,
                uid,
                submit_time: DateTime::now(),
                judge_time: None,
                report: None,
                lang: None,
                status: None,
                time: None,
                memory: None,
            };
            data.insert(id, r.clone());
            Ok(r)
        }
    }
}
