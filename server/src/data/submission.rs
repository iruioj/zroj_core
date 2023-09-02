use super::{error::DataError, types::*};
use crate::{manager::problem_judger::FullJudgeReport, ProblemID, SubmID, UserID};
use async_trait::async_trait;
#[cfg(feature = "mysql")]
use diesel::*;
use judger::StoreFile;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::collections::BTreeMap;

pub struct SubmRaw(pub BTreeMap<String, StoreFile>);

/// 提交记录的元信息，不包含源代码、评测日志等
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "mysql", derive(Queryable, AsChangeset))]
#[cfg_attr(feature = "mysql", diesel(table_name = database::submissions))]
pub struct Submission {
    pub id: SubmID,
    pub pid: ProblemID,
    pub uid: UserID,
    pub submit_time: DateTime,
    pub judge_time: Option<DateTime>,
    pub size: CastMemory,
    /// 不是每一个提交记录都有确定的源文件语言
    pub lang: Option<JsonStr<judger::FileType>>,
    /// 评测状态，None 表示暂无（不一定是评测中）
    pub status: Option<JsonStr<judger::Status>>,
    /// 所有测试点中消耗时间的最大值
    pub time: Option<CastElapse>,
    /// 所有测试点中占用内存的最大值
    pub memory: Option<CastMemory>,
    /// 评测结果
    pub report: Option<JsonStr<FullJudgeReport>>,
}

#[derive(TsType, Serialize)]
pub struct SubmMeta {
    id: SubmID,
    pid: ProblemID,
    uid: UserID,
    submit_time: String,
    judge_time: Option<String>,
    lang: Option<judger::FileType>,
    status: Option<judger::Status>,
    time: Option<problem::Elapse>,
    memory: Option<problem::Memory>,
}

#[derive(TsType, Serialize)]
pub struct SubmInfo {
    meta: SubmMeta,
    report: Option<FullJudgeReport>,
}

impl From<Submission> for SubmMeta {
    fn from(value: Submission) -> Self {
        Self {
            id: value.id,
            pid: value.pid,
            uid: value.uid,
            submit_time: value.submit_time.to_string(),
            judge_time: value.judge_time.map(|s| s.to_string()),
            lang: value.lang.map(|o| o.0),
            status: value.status.map(|o| o.0),
            time: value.time.map(|o| o.0),
            memory: value.memory.map(|o| o.0),
        }
    }
}
impl From<Submission> for SubmInfo {
    fn from(value: Submission) -> Self {
        Self {
            meta: SubmMeta {
                id: value.id,
                pid: value.pid,
                uid: value.uid,
                submit_time: value.submit_time.to_string(),
                judge_time: value.judge_time.map(|s| s.to_string()),
                lang: value.lang.map(|o| o.0),
                status: value.status.map(|o| o.0),
                time: value.time.map(|o| o.0),
                memory: value.memory.map(|o| o.0),
            },
            report: value.report.map(|o| o.0),
        }
    }
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
            size -> Unsigned<BigInt>,
            lang -> Nullable<Text>,
            status -> Nullable<Text>,
            time -> Nullable<Unsigned<BigInt>>,
            memory -> Nullable<Unsigned<BigInt>>,
            report -> Nullable<Text>,
        }
    }
}

#[derive(Default, TsType, Deserialize)]
pub struct FilterOption {
    // status: Option<judger::Status>,
    pid: Option<ProblemID>,
    uid: Option<UserID>,
    lang: Option<judger::FileType>,
}

pub type SubmDB = dyn Manager + Sync + Send;

#[async_trait(?Send)]
pub trait Manager {
    async fn insert_new(
        &self,
        uid: UserID,
        pid: ProblemID,
        raw: &mut SubmRaw,
    ) -> Result<Submission, DataError>;
    async fn get(&self, sid: &SubmID) -> Result<Submission, DataError>;
    async fn get_raw(&self, sid: &SubmID) -> Result<SubmRaw, DataError>;
    async fn update(&self, sid: &SubmID, report: FullJudgeReport) -> Result<(), DataError>;
    /// get submission meta list
    async fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        filter: FilterOption,
    ) -> Result<Vec<SubmMeta>, DataError>;
}

mod default {
    use std::{
        collections::{btree_map::Entry, BTreeMap},
        sync::RwLock,
    };

    use store::{FsStore, Handle};

    use crate::SubmID;

    use super::*;

    pub struct DefaultDB {
        data: RwLock<BTreeMap<SubmID, Submission>>,
        dir: RwLock<Handle>,
    }

    impl DefaultDB {
        pub fn new(dir: impl AsRef<std::path::Path>) -> Self {
            std::fs::create_dir_all(dir.as_ref()).expect("create subm db dir");
            Self {
                data: Default::default(),
                dir: RwLock::new(Handle::new(dir.as_ref())),
            }
        }
    }

    #[async_trait(?Send)]
    impl super::Manager for DefaultDB {
        async fn insert_new(
            &self,
            uid: UserID,
            pid: ProblemID,
            raw: &mut SubmRaw,
        ) -> Result<Submission, DataError> {
            let mut data = self.data.write()?;
            let id = data.iter().next_back().map(|x| *x.0).unwrap_or(0) + 1;
            let size = raw
                .0
                .iter()
                .fold(0, |acc, cur| acc + cur.1.file.metadata().unwrap().len());

            let g = self.dir.write()?;
            let raw_dir = g.join(id.to_string());
            FsStore::save(&mut raw.0, &raw_dir)?;
            drop(g);

            let r = Submission {
                id,
                pid,
                uid,
                submit_time: DateTime::now(),
                size: CastMemory(problem::Memory::from(size)),
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
        async fn get(&self, sid: &SubmID) -> Result<Submission, DataError> {
            self.data
                .read()?
                .get(sid)
                .cloned()
                .ok_or(DataError::NotFound)
        }
        async fn update(&self, sid: &SubmID, report: FullJudgeReport) -> Result<(), DataError> {
            let mut binding = self.data.write()?;
            let Entry::Occupied(mut e) = binding.entry(*sid) else {
                return Err(DataError::NotFound)
            };
            let entry = e.get_mut();
            entry.report = Some(JsonStr(report));
            Ok(())
        }
        async fn get_raw(&self, sid: &SubmID) -> Result<SubmRaw, DataError> {
            let g = self.dir.read()?;
            let raw_dir = g.join(sid.to_string());
            let r = FsStore::open(&raw_dir)?;
            drop(g);
            Ok(SubmRaw(r))
        }
        async fn get_metas(
            &self,
            max_count: u8,
            offset: usize,
            filter: FilterOption,
        ) -> Result<Vec<SubmMeta>, DataError> {
            let db = self.data.read()?;
            let f_uid = |uid: &UserID| filter.uid.as_ref().map(|u| u == uid).unwrap_or(true);
            let f_pid = |uid: &UserID| filter.pid.as_ref().map(|u| u == uid).unwrap_or(true);
            let f_lang =
                |x: &judger::FileType| filter.lang.as_ref().map(|y| y == x).unwrap_or(true);
            // let f_status =
            //     |st: &judger::Status| filter.status.as_ref().map(|s| s == st).unwrap_or(true);
            let data = db
                .iter()
                .filter(|(_, m)| {
                    f_uid(&m.uid)
                        && f_pid(&m.pid)
                        && m.lang.as_ref().map(|v| f_lang(&v.0)).unwrap_or(false)
                })
                .skip(offset)
                .take(max_count.into())
                .map(|(_, m)| SubmMeta::from(m.clone()));
            Ok(data.collect())
        }
    }
}
pub use default::DefaultDB;
