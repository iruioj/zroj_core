use super::{
    error::DataError,
    mysql::{MysqlConfig, MysqlDb, last_insert_id, schema_model::ProblemStatement},
};
use crate::{
    data::{mysql::schema::problem_statements, types::JsonStr},
    ProblemID,
};
use actix_files::NamedFile;
use diesel::*;
use problem::render_data::{self, statement::StmtMeta, Mdast};
use serde::Serialize;
use serde_ts_typing::TsType;
use std::sync::RwLock;
use store::Handle;

pub type StmtDB = dyn Manager + Sync + Send;

#[derive(Debug, Insertable)]
#[diesel(table_name = problem_statements)]
pub struct NewProblemStatement<'a> {
    title: &'a String,
    content: &'a JsonStr<Mdast>,
    meta: &'a JsonStr<StmtMeta>,
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

#[derive(Debug, Serialize, TsType)]
pub struct StmtMetaDisplay {
    pid: ProblemID,
    title: String,
    meta: StmtMeta,
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
    ) -> Result<Vec<StmtMetaDisplay>, DataError>;
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
            let r: ProblemStatement = problem_statements::table
                .filter(problem_statements::pid.eq(id))
                .first(conn)?;
            Ok(Statement {
                title: r.title,
                statement: r.content.0,
                meta: r.meta.0,
            })
        })
    }

    fn insert_new(&self, stmt: render_data::Statement) -> Result<ProblemID, DataError> {
        self.0.transaction(|conn| {
            let val = NewProblemStatement {
                title: &stmt.title,
                content: &JsonStr(stmt.statement.render_mdast()),
                meta: &JsonStr(stmt.meta),
            };
            diesel::insert_into(problem_statements::table)
                .values(&val)
                .execute(conn)?;
            // https://github.com/diesel-rs/diesel/issues/1011
            let pid: u64 = diesel::select(last_insert_id()).first(conn)?;
            Ok(pid as ProblemID)
        })
    }
    fn update(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            let val = ProblemStatement {
                pid: id,
                title: stmt.title,
                content: JsonStr(stmt.statement.render_mdast()),
                meta: JsonStr(stmt.meta),
            };
            diesel::replace_into(problem_statements::table)
                .values(&val)
                .execute(conn)?;
            Ok(())
        })
    }

    fn get_assets(&self, id: ProblemID, name: &str) -> Result<NamedFile, DataError> {
        Ok(NamedFile::open(
            self.1.read()?.join(id.to_string()).join(name).as_ref(),
        )?)
    }

    fn insert_assets(
        &self,
        id: ProblemID,
        mut file: std::fs::File,
        name: &str,
    ) -> Result<(), DataError> {
        let path = self.1.write()?.join(id.to_string()).join(name);
        path.remove_all()?;
        let mut dest = path.create_new_file()?;
        std::io::copy(&mut file, &mut dest)?;
        Ok(())
    }

    fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<StmtMetaDisplay>, DataError> {
        self.0.transaction(|conn| {
            let r = problem_statements::table
                .select((
                    problem_statements::pid,
                    problem_statements::title,
                    problem_statements::meta,
                ))
                .filter(problem_statements::title.like(pattern.unwrap_or("%".into())))
                .offset(offset as i64)
                .limit(max_count as i64)
                .load::<(ProblemID, String, JsonStr<StmtMeta>)>(conn)?;
            Ok(r.into_iter()
                .map(|(pid, title, meta)| StmtMetaDisplay {
                    pid,
                    title,
                    meta: meta.0,
                })
                .collect())
        })
    }
}
