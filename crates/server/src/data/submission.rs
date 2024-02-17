use super::{
    error::DataError,
    mysql::{
        last_insert_id,
        schema::*,
        schema_model::{ContestSubmission, SubmissionDetail, SubmissionMeta},
        MysqlDb,
    },
    types::*,
};
use crate::{CtstID, ProblemID, SubmID, UserID};
use diesel::*;
use serde::Serialize;
use serde_ts_typing::TsType;

#[derive(TsType, Serialize)]
pub struct SubmMeta {
    id: SubmID,
    pub pid: ProblemID,
    problem_title: String,
    uid: UserID,
    username: Username,
    submit_time: String,
    judge_time: Option<String>,
    lang: Option<judger::FileType>,
    status: Option<judger::Status>,
    time: Option<problem::Elapse>,
    memory: Option<problem::Memory>,
}

impl From<(SubmissionMeta, String, Username)> for SubmMeta {
    fn from((s, problem_title, username): (SubmissionMeta, String, Username)) -> Self {
        Self {
            id: s.id,
            pid: s.pid,
            problem_title,
            uid: s.uid,
            username,
            submit_time: s.submit_time.to_string(),
            judge_time: s.judge_time.map(|o| o.to_string()),
            lang: s.lang.map(|o| o.0),
            status: s.status.map(|o| o.0),
            time: s.time.map(|o| o.0),
            memory: s.memory.map(|o| o.0),
        }
    }
}
#[derive(TsType, Serialize)]
pub struct SubmInfo {
    pub meta: SubmMeta,
    pub raw: SubmRaw,
    report: Option<FullJudgeReport>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = submission_metas)]
pub struct NewSubmissionMeta {
    pid: ProblemID,
    uid: UserID,
    submit_time: DateTime,
    lang: Option<JsonStr<judger::FileType>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = submission_details)]
struct NewSubmissionDetail {
    sid: SubmID,
    raw: SubmRaw,
}

pub struct SubmDB(MysqlDb);

impl SubmDB {
    pub fn new(mysqldb: &MysqlDb) -> Self {
        Self(mysqldb.clone())
    }
    pub fn insert_new(
        &self,
        uid: UserID,
        pid: ProblemID,
        cid: Option<CtstID>,
        lang: Option<judger::FileType>,
        raw: &SubmRaw,
    ) -> Result<SubmID, DataError> {
        self.0.transaction(|conn| {
            diesel::insert_into(submission_metas::table)
                .values(NewSubmissionMeta {
                    uid,
                    pid,
                    lang: lang.map(JsonStr),
                    submit_time: DateTime::now(),
                })
                .execute(conn)?;

            let id: u64 = diesel::select(last_insert_id()).first(conn)?;
            diesel::insert_into(submission_details::table)
                .values(NewSubmissionDetail {
                    sid: id as SubmID,
                    raw: raw.clone(),
                })
                .execute(conn)?;
            if let Some(cid) = cid {
                diesel::insert_into(contest_submissions::table)
                    .values(ContestSubmission {
                        cid,
                        sid: id as SubmID,
                    })
                    .execute(conn)?;
            }
            Ok(id as SubmID)
        })
    }

    pub fn get_info(&self, sid: &SubmID) -> Result<SubmInfo, DataError> {
        self.0.transaction(|conn| {
            let meta: SubmissionMeta = submission_metas::table
                .filter(submission_metas::id.eq(*sid))
                .first(conn)?;
            let detail: SubmissionDetail = SubmissionDetail::belonging_to(&meta).first(conn)?;
            let title: String = problems::table
                .select(problems::title)
                .filter(problems::id.eq(meta.pid))
                .first(conn)?;
            let username: Username = users::table
                .select(users::username)
                .filter(users::id.eq(meta.uid))
                .first(conn)?;
            Ok(SubmInfo {
                meta: SubmMeta::from((meta, title, username)),
                raw: detail.raw,
                report: detail.report,
            })
        })
    }

    pub fn update(&self, sid: &SubmID, report: FullJudgeReport) -> Result<(), DataError> {
        let memory = CastMemory(report.max_memory());
        let time = CastElapse(report.max_time());
        let status = report.status().map(JsonStr);

        self.0.transaction(|conn| {
            diesel::update(submission_details::table.filter(submission_details::sid.eq(*sid)))
                .set(submission_details::report.eq(report))
                .execute(conn)?;
            diesel::update(submission_metas::table.filter(submission_metas::id.eq(*sid)))
                .set((
                    submission_metas::judge_time.eq(DateTime::now()),
                    submission_metas::memory.eq(memory),
                    submission_metas::time.eq(time),
                    submission_metas::status.eq(status),
                ))
                .execute(conn)?;
            Ok(())
        })
    }

    pub fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pid: Option<ProblemID>,
        uid: Option<UserID>,
        cid: Option<CtstID>,
        lang: Option<judger::FileType>,
    ) -> Result<Vec<SubmMeta>, DataError> {
        self.0.transaction(|conn| {
            let mut table = submission_metas::table
                .into_boxed()
                .offset(offset as i64)
                .limit(max_count as i64);
            if let Some(pid) = pid {
                table = table.filter(submission_metas::pid.eq(pid));
            }
            if let Some(uid) = uid {
                table = table.filter(submission_metas::uid.eq(uid));
            }
            if let Some(lang) = lang {
                table = table.filter(submission_metas::lang.eq(JsonStr(lang)));
            }
            let table = table
                .inner_join(problems::table)
                .inner_join(users::table)
                .left_join(contest_submissions::table);
            let res: Vec<(SubmissionMeta, String, Username)> = if let Some(cid) = cid {
                table
                    .filter(contest_submissions::cid.eq(cid))
                    .select((
                        SubmissionMeta::as_select(),
                        problems::title,
                        users::username,
                    ))
                    .load::<(SubmissionMeta, String, Username)>(conn)
            } else {
                table
                    .select((
                        SubmissionMeta::as_select(),
                        problems::title,
                        users::username,
                    ))
                    .load::<(SubmissionMeta, String, Username)>(conn)
            }?;
            Ok(res.into_iter().map(SubmMeta::from).collect())
        })
    }
}
