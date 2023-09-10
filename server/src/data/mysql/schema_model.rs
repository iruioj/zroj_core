//! 对 schema 的基础 model，一个数据类型对应一个 table

use super::super::types::*;
use super::schema::*;
use crate::{ProblemID, UserID, SubmID};
use diesel::*;
use problem::render_data::{statement::StmtMeta, Mdast};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Identifiable, Clone, Queryable, AsChangeset)]
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

#[derive(Debug, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name = problem_statements)]
pub struct ProblemStatement {
    pub pid: ProblemID,
    pub title: String,
    pub content: JsonStr<Mdast>,
    pub meta: JsonStr<StmtMeta>,
}


/// 提交记录的元信息
#[derive(Debug, Serialize, Deserialize, Clone, Identifiable, Queryable, AsChangeset, Selectable)]
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
    // 评测结果
    // report: Option<FullJudgeReport>,
}

#[derive(Debug, Serialize, Deserialize, Associations, Identifiable, Clone, Queryable, AsChangeset, Selectable)]
#[diesel(belongs_to(SubmissionMeta, foreign_key = sid))]
#[diesel(table_name = submission_details)]
pub struct SubmissionDetail {
    pub id: u32, // not important
    pub sid: SubmID,
    pub raw: SubmRaw,
    pub report: Option<FullJudgeReport>,
}