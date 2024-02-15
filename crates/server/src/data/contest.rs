use crate::{CtstID, UserID};

use super::{
    error::DataError,
    mysql::{
        schema::{contest_problems, contest_registrants, contests, problems, users},
        schema_model::{Contest, ContestRegistrant, Problem},
        MysqlDb,
    },
    problem_statement::ProblemMeta,
    types::*,
};
use diesel::*;
use serde::Serialize;
use serde_ts_typing::TsType;

pub struct CtstDB(MysqlDb);

impl CtstDB {
    pub fn new(mysqldb: &MysqlDb) -> Self {
        Self(mysqldb.clone())
    }
    pub fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<ContestMeta>, DataError> {
        self.0.transaction(|conn| {
            Ok(contests::table
                .filter(
                    contests::title.like(
                        pattern
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("%".into()),
                    ),
                )
                .offset(offset as i64)
                .limit(max_count as i64)
                .load::<Contest>(conn)?
                .into_iter()
                .map(ContestMeta::from)
                .collect())
        })
    }
    pub fn get(&self, id: CtstID) -> Result<ContestInfo, DataError> {
        self.0.transaction(|conn| {
            let meta: Contest = contests::table.filter(contests::id.eq(id)).first(conn)?;
            let meta: ContestMeta = meta.into();
            let problems: Vec<ProblemMeta> = contest_problems::table
                .filter(contest_problems::cid.eq(id))
                .inner_join(problems::table)
                .select(Problem::as_select())
                .load(conn)?
                .into_iter()
                .map(Problem::into)
                .collect();

            Ok(ContestInfo { meta, problems })
        })
    }
    pub fn insert_registrant(&self, id: CtstID, uid: UserID) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            diesel::insert_or_ignore_into(contest_registrants::table)
                .values(&ContestRegistrant { cid: id, uid })
                .execute(conn)?;
            Ok(())
        })
    }
    pub fn remove_registrant(&self, id: CtstID, uid: UserID) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            diesel::delete(
                contest_registrants::table.filter(
                    contest_registrants::cid
                        .eq(id)
                        .and(contest_registrants::uid.eq(uid)),
                ),
            )
            .execute(conn)?;
            Ok(())
        })
    }
    pub fn get_registrants(
        &self,
        id: CtstID,
        max_count: u8,
        offset: usize,
    ) -> Result<Vec<UserMeta>, DataError> {
        self.0.transaction(|conn| {
            let user_metas: Vec<UserMeta> = contest_registrants::table
                .filter(contest_registrants::cid.eq(id))
                .inner_join(users::table)
                .offset(offset as i64)
                .limit(max_count as i64)
                .select((users::id, users::username))
                .load::<(UserID, Username)>(conn)?
                .into_iter()
                .map(|(id, username)| UserMeta { id, username })
                .collect();

            Ok(user_metas)
        })
    }
}

#[derive(TsType, Serialize)]
pub struct ContestMeta {
    pub id: CtstID,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration: CastElapse,
}

impl From<Contest> for ContestMeta {
    fn from(c: Contest) -> Self {
        ContestMeta {
            id: c.id,
            title: c.title,
            start_time: c.start_time.to_i64(),
            end_time: c.end_time.to_i64(),
            duration: c.duration,
        }
    }
}

#[derive(Serialize, TsType)]
pub struct ContestInfo {
    meta: ContestMeta,
    problems: Vec<ProblemMeta>,
}

#[derive(Serialize, TsType)]
pub struct UserMeta {
    id: UserID,
    username: Username,
}
