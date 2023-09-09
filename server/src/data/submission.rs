use super::{error::DataError, types::*};
use crate::{manager::problem_judger::FullJudgeReport, ProblemID, SubmID, UserID};
use async_trait::async_trait;
use diesel::*;
use judger::StoreFile;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::collections::BTreeMap;

pub struct SubmRaw(pub BTreeMap<String, StoreFile>);

const SOURCE_LIMIT: usize = 100 * 1024;

impl SubmRaw {
    pub fn to_display_vec(
        self,
    ) -> Result<Vec<(String, judger::FileType, judger::truncstr::TruncStr)>, DataError> {
        self.0
            .into_iter()
            .filter(|(_, v)| !matches!(v.file_type, judger::FileType::Binary))
            .map(|(k, mut v)| {
                Ok((
                    k,
                    v.file_type.clone(),
                    judger::truncstr::TruncStr::new(v.read_to_string()?, SOURCE_LIMIT),
                ))
            })
            .collect::<Result<Vec<_>, std::io::Error>>()
            .map_err(DataError::IO)
    }
}

/// 提交记录的元信息，不包含源代码、评测日志等
#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Queryable, AsChangeset)]
#[diesel(table_name = database::submissions)]
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
    problem_title: String,
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

pub type SubmDB = dyn Manager + Sync + Send;

#[async_trait(?Send)]
pub trait Manager {
    async fn insert_new(
        &self,
        uid: UserID,
        pid: ProblemID,
        lang: Option<judger::FileType>,
        raw: &mut SubmRaw,
    ) -> Result<Submission, DataError>;
    async fn get_info(&self, sid: &SubmID) -> Result<SubmInfo, DataError>;
    async fn get_raw(&self, sid: &SubmID) -> Result<SubmRaw, DataError>;
    async fn update(&self, sid: &SubmID, report: FullJudgeReport) -> Result<(), DataError>;
    /// get submission meta list
    async fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pid: Option<ProblemID>,
        uid: Option<UserID>,
        lang: Option<judger::FileType>,
    ) -> Result<Vec<SubmMeta>, DataError>;
}

mod default {
    use super::*;
    use crate::{data::fs_store::FsStoreDb, SubmID};
    use std::collections::{btree_map::Entry, BTreeMap};
    use store::FsStore;

    #[derive(FsStore, Default)]
    pub struct Data {
        #[meta]
        data: BTreeMap<SubmID, Submission>,
    }
    pub struct DefaultDB(FsStoreDb);

    impl DefaultDB {
        pub fn new(dir: impl AsRef<std::path::Path>) -> Self {
            Self(FsStoreDb::new(dir))
        }
    }

    #[async_trait(?Send)]
    impl super::Manager for DefaultDB {
        async fn insert_new(
            &self,
            uid: UserID,
            pid: ProblemID,
            lang: Option<judger::FileType>,
            raw: &mut SubmRaw,
        ) -> Result<Submission, DataError> {
            let db = self.0.table::<Data>().get_table()?;
            let mut dbw = db.write()?;
            let id = dbw.data.iter().next_back().map(|x| *x.0).unwrap_or(0) + 1;
            let size = raw
                .0
                .iter()
                .fold(0, |acc, cur| acc + cur.1.file.metadata().unwrap().len());

            let raw_dir: store::Handle = db.ctx.join(id.to_string());
            FsStore::save(&mut raw.0, &raw_dir)?;

            let r = Submission {
                id,
                pid,
                uid,
                submit_time: DateTime::now(),
                size: CastMemory(problem::Memory::from(size)),
                judge_time: None,
                report: None,
                lang: lang.map(JsonStr),
                status: None,
                time: None,
                memory: None,
            };
            dbw.data.insert(id, r.clone());
            Ok(r)
        }
        async fn get_info(&self, sid: &SubmID) -> Result<SubmInfo, DataError> {
            todo!()
        }
        async fn update(&self, sid: &SubmID, report: FullJudgeReport) -> Result<(), DataError> {
            self.0.table::<Data>().write_transaction(|dbw| {
                let Entry::Occupied(mut e) = dbw.data.entry(*sid) else {
                    return Err(DataError::NotFound)
                };
                let entry = e.get_mut();
                entry.report = Some(JsonStr(report));
                Ok(())
            })
        }
        async fn get_raw(&self, sid: &SubmID) -> Result<SubmRaw, DataError> {
            let db = self.0.table::<Data>().get_table()?;
            let raw_dir = db.ctx.join(sid.to_string());
            let r = FsStore::open(&raw_dir)?;
            Ok(SubmRaw(r))
        }
        async fn get_metas(
            &self,
            _max_count: u8,
            _offset: usize,
            _pid: Option<ProblemID>,
            _uid: Option<UserID>,
            _lang: Option<judger::FileType>,
        ) -> Result<Vec<SubmMeta>, DataError> {
            todo!()
            // let db = self.data.read()?;
            // let f_uid = |x: &UserID| uid.as_ref().map(|u| u == x).unwrap_or(true);
            // let f_pid = |x: &UserID| pid.as_ref().map(|u| u == x).unwrap_or(true);
            // let f_lang = |x: Option<&judger::FileType>| lang.is_none() || lang.as_ref() == x;
            // let data = db
            //     .iter()
            //     .filter(|(_, m)| {
            //         f_uid(&m.uid) && f_pid(&m.pid) && f_lang(m.lang.as_ref().map(|v| &v.0))
            //     })
            //     .skip(offset)
            //     .take(max_count.into())
            //     .map(|(_, m)| SubmMeta::from(m.clone()));
            // Ok(data.collect())
        }
    }
}
pub use default::DefaultDB;
