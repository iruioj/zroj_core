use super::{
    error::DataError,
    mysql::{
        last_insert_id,
        schema::problems,
        schema_model::{Problem, ProblemStatement},
        MysqlConfig, MysqlDb,
    },
};
use crate::{
    data::{mysql::schema::problem_statements, types::JsonStr},
    ProblemID,
};
use actix_files::NamedFile;
use anyhow::Context;
use diesel::*;
use problem::render_data::{self, statement::StmtMeta, Mdast};
use serde::Serialize;
use serde_ts_typing::TsType;
use std::sync::RwLock;
use store::Handle;

pub type StmtDB = dyn Manager + Sync + Send;

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

pub trait Manager {
    /// HTML statement
    fn get(&self, id: ProblemID) -> Result<Statement, DataError>;
    /// parse statement for reader and insert (update) it
    fn insert_new(&self, stmt: render_data::Statement) -> Result<ProblemID, DataError>;
    fn update(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), DataError>;
    fn get_assets(&self, id: ProblemID, name: &str) -> Result<NamedFile, DataError>;
    fn insert_assets(
        &self,
        id: ProblemID,
        file: std::fs::File,
        name: &str,
    ) -> Result<(), DataError>;
    /// get problem meta list, often used for problem listing
    fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<ProblemMeta>, DataError>;
}

pub struct Mysql(MysqlDb, RwLock<Handle>);

impl Mysql {
    /// note that assert directory should be only accessible by
    /// this database, so you pass its ownership to it
    pub fn new(cfg: &MysqlConfig, asset_dir: Handle) -> Self {
        Self(MysqlDb::new(cfg), RwLock::new(asset_dir))
    }
}

impl Manager for Mysql {
    fn get(&self, id: ProblemID) -> Result<Statement, DataError> {
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

    fn insert_new(&self, stmt: render_data::Statement) -> Result<ProblemID, DataError> {
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
    fn update(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), DataError> {
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

    fn get_assets(&self, id: ProblemID, name: &str) -> Result<NamedFile, DataError> {
        Ok(
            NamedFile::open(self.1.read()?.join(id.to_string()).join(name).as_ref())
                .context("open asset file")?,
        )
    }

    fn insert_assets(
        &self,
        id: ProblemID,
        mut file: std::fs::File,
        name: &str,
    ) -> Result<(), DataError> {
        let path = self.1.write()?.join(id.to_string()).join(name);
        path.remove_all().context("remove asset path")?;
        let mut dest = path.create_new_file().context("create new asset dest")?;
        std::io::copy(&mut file, &mut dest).context("copy asset file")?;
        Ok(())
    }

    fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<ProblemMeta>, DataError> {
        self.0.transaction(|conn| {
            Ok(problems::table
                .filter(problems::title.like(pattern.filter(|s| s.trim().len() > 0).unwrap_or("%".into())))
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
