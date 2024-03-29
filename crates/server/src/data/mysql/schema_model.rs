//! Define rust types modeling each table schema.

use super::super::types::*;
use super::schema::*;
use crate::{CtstID, ProblemID, SubmID, UserID};
use diesel::*;
use problem::render_data::{statement::StmtMeta, Mdast};

#[derive(Debug, Identifiable, Clone, Queryable, AsChangeset, Selectable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    /// 用户 id
    pub id: UserID,
    /// 用户名
    pub username: Username,
    /// 密码的 hash 值
    pub password_hash: String,
    /// 真实姓名
    pub name: String,
    /// 邮箱
    pub email: EmailAddress,
    /// 格言
    pub motto: String,
    /// 注册时间
    pub register_time: DateTime,
    /// 性别
    pub gender: Gender,
}

#[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Selectable, Insertable)]
#[diesel(table_name = problems)]
pub struct Problem {
    pub id: ProblemID,
    pub title: String,
    pub meta: JsonStr<StmtMeta>,
}

#[derive(
    Debug, Clone, Queryable, Associations, Identifiable, AsChangeset, Selectable, Insertable,
)]
#[diesel(belongs_to(Problem, foreign_key = pid))]
#[diesel(table_name = problem_statements)]
pub struct ProblemStatement {
    pub id: u32, // useless
    pub pid: ProblemID,
    pub content: JsonStr<Mdast>,
}

/// 提交记录的元信息
#[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Selectable, Insertable)]
#[diesel(table_name = submission_metas)]
// #[diesel(belongs_to(User, foreign_key = uid))]
// #[diesel(belongs_to(ProblemStatement, foreign_key = pid))]
pub struct SubmissionMeta {
    pub id: SubmID,
    pub pid: ProblemID,
    pub uid: UserID,
    pub submit_time: DateTime,
    // raw: SubmRaw,
    pub judge_time: Option<DateTime>,
    /// 不是每一个提交记录都有确定的源文件语言
    pub lang: Option<JsonStr<judger::FileType>>,
    /// 评测状态，None 表示暂无（不一定是评测中）
    pub status: Option<JsonStr<judger::Status>>,
    /// 所有测试点中消耗时间的最大值
    pub time: Option<CastElapse>,
    /// 所有测试点中占用内存的最大值
    pub memory: Option<CastMemory>,
}

#[derive(Debug, Associations, Identifiable, Clone, Queryable, AsChangeset, Selectable)]
#[diesel(belongs_to(SubmissionMeta, foreign_key = sid))]
#[diesel(table_name = submission_details)]
pub struct SubmissionDetail {
    pub id: u32, // not important
    pub sid: SubmID,
    pub raw: SubmRaw,
    pub report: Option<FullJudgeReport>,
}

#[derive(Debug, Identifiable, Clone, Queryable, AsChangeset, Selectable, Insertable)]
#[diesel(table_name = contests)]
pub struct Contest {
    pub id: CtstID,
    pub title: String,
    /// contest start time is not necessarily contestants' start_time
    pub start_time: DateTime,
    /// contestants' end time must be earlier than contest end time
    pub end_time: DateTime,
    pub duration: CastElapse,
}

#[derive(Debug, Associations, Identifiable, Clone, Queryable, Selectable, Insertable)]
#[diesel(belongs_to(Contest, foreign_key = cid))]
#[diesel(belongs_to(Problem, foreign_key = pid))]
#[diesel(table_name = contest_problems)]
#[diesel(primary_key(cid, pid))]
pub struct ContestProblem {
    pub cid: CtstID,
    pub pid: ProblemID,
}

#[derive(Debug, Associations, Identifiable, Clone, Queryable, Selectable, Insertable)]
#[diesel(belongs_to(Contest, foreign_key = cid))]
#[diesel(belongs_to(User, foreign_key = uid))]
#[diesel(table_name = contest_registrants)]
#[diesel(primary_key(cid, uid))]
pub struct ContestRegistrant {
    pub cid: CtstID,
    pub uid: UserID,
    /// This contest is available for the registrant starting from
    /// max([`Contest::start_time`], [`ContestRegistrant::register_time`]),
    /// elapsing [`Contest::duration`] but not exceeding [`Contest::end_time`].
    pub register_time: DateTime,
}

#[derive(Debug, Associations, Identifiable, Clone, Queryable, Selectable, Insertable)]
#[diesel(belongs_to(Contest, foreign_key = cid))]
#[diesel(belongs_to(SubmissionMeta, foreign_key = sid))]
#[diesel(table_name = contest_submissions)]
#[diesel(primary_key(cid, sid))]
pub struct ContestSubmission {
    pub cid: CtstID,
    pub sid: SubmID,
}
