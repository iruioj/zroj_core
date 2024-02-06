use super::{
    error::DataError,
    file_system::{FileSysDb, FileSysTable, SanitizeError, SanitizedString},
    mysql::{
        last_insert_id,
        schema::problems,
        schema_model::{Problem, ProblemStatement},
        MysqlDb,
    },
};
use crate::{
    data::{file_system::schema::staticdata, mysql::schema::problem_statements, types::JsonStr},
    ProblemID,
};
use actix_files::NamedFile;
use anyhow::Context;
use diesel::*;
use problem::render_data::{self, statement::StmtMeta, Mdast};
use serde::Serialize;
use serde_ts_typing::TsType;

#[derive(Debug, Insertable)]
#[diesel(table_name = problems)]
struct NewProblem<'a> {
    title: &'a String,
    meta: &'a JsonStr<StmtMeta>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = problem_statements)]
struct NewProblemStatement<'a> {
    pid: ProblemID,
    content: &'a JsonStr<Mdast>,
}

#[derive(Debug, Serialize, TsType)]
pub struct Statement {
    title: String,
    statement: Mdast,
    meta: StmtMeta,
}

impl From<&render_data::Statement> for Statement {
    fn from(value: &render_data::Statement) -> Self {
        Self {
            title: value.title.clone(),
            statement: value.statement.render_mdast(),
            meta: value.meta.clone(),
        }
    }
}

#[derive(Serialize, TsType)]
pub struct ProblemMeta {
    pub id: ProblemID,
    pub title: String,
    pub tags: String,
}

pub type StmtDB = Mysql;

pub struct Mysql(MysqlDb, FileSysDb);

impl Mysql {
    /// note that assert directory should be only accessible by
    /// this database, so you pass its ownership to it
    pub fn new(mysqldb: &MysqlDb, filesysdb: &FileSysDb) -> Self {
        Self(mysqldb.clone(), filesysdb.clone())
    }
}

impl Mysql {
    pub fn get(&self, id: ProblemID) -> Result<Statement, DataError> {
        self.0.transaction(|conn| {
            let problem: Problem = problems::table.filter(problems::id.eq(id)).first(conn)?;
            let statement: ProblemStatement =
                ProblemStatement::belonging_to(&problem).first(conn)?;
            Ok(Statement {
                title: problem.title,
                statement: statement.content.0,
                meta: problem.meta.0,
            })
        })
    }

    pub fn insert_new(&self, stmt: render_data::Statement) -> Result<ProblemID, DataError> {
        self.0.transaction(|conn| {
            diesel::insert_into(problems::table)
                .values(NewProblem {
                    title: &stmt.title,
                    meta: &JsonStr(stmt.meta),
                })
                .execute(conn)?;
            // https://github.com/diesel-rs/diesel/issues/1011
            let pid: u64 = diesel::select(last_insert_id()).first(conn)?;
            diesel::insert_into(problem_statements::table)
                .values(NewProblemStatement {
                    pid: pid as ProblemID,
                    content: &JsonStr(stmt.statement.render_mdast()),
                })
                .execute(conn)?;
            Ok(pid as ProblemID)
        })
    }
    pub fn update(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            diesel::update(problem_statements::table.filter(problem_statements::pid.eq(id)))
                .set(problem_statements::content.eq(JsonStr(stmt.statement.render_mdast())))
                .execute(conn)?;
            diesel::update(problems::table.filter(problems::id.eq(id)))
                .set((
                    problems::title.eq(stmt.title),
                    problems::meta.eq(JsonStr(stmt.meta)),
                ))
                .execute(conn)?;
            Ok(())
        })
    }

    fn build_key(&self, id: ProblemID, name: &str) -> Result<SanitizedString, SanitizeError> {
        SanitizedString::new(&format!("problem_statement/{id}/{name}"))
    }

    fn get_assets(&self, id: ProblemID, name: &str) -> Result<NamedFile, DataError> {
        let key = self.build_key(id, name)?;
        self.1.transaction(|ctx| {
            let (file, ctx) = staticdata::conn(ctx).query_with_ctx(&key)?;
            Ok(NamedFile::from_file(file, ctx.path()).context("open asset file")?)
        })
    }

    fn insert_assets(
        &self,
        id: ProblemID,
        mut file: std::fs::File,
        name: &str,
    ) -> Result<(), DataError> {
        let key = self.build_key(id, name)?;
        self.1
            .transaction(|ctx| staticdata::conn(ctx).replace(&key, &mut file))
    }

    pub fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<ProblemMeta>, DataError> {
        self.0.transaction(|conn| {
            Ok(problems::table
                .filter(
                    problems::title.like(
                        pattern
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("%".into()),
                    ),
                )
                .offset(offset as i64)
                .limit(max_count as i64)
                .load::<Problem>(conn)?
                .into_iter()
                .map(|p| ProblemMeta {
                    id: p.id,
                    title: p.title,
                    tags: "umimplemented".into(),
                })
                .collect())
        })
    }
}
