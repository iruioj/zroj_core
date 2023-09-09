use super::{
    error::DataError,
    mysql::{MysqlConfig, MysqlDb},
};
use crate::{
    data::{mysql::schema::problem_statements, types::JsonStr},
    ProblemID,
};
use actix_files::NamedFile;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, *};
use problem::render_data::{self, statement::StmtMeta, Mdast};
use serde::Serialize;
use serde_ts_typing::TsType;
use std::sync::RwLock;
use store::Handle;

pub type StmtDB = dyn Manager + Sync + Send;

#[derive(Debug, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name = problem_statements)]
struct ProblemStatement {
    pid: ProblemID,
    title: String,
    content: JsonStr<Mdast>,
    meta: JsonStr<StmtMeta>,
}

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

// use diesel::sql_types::{Integer, Unsigned};
sql_function! { fn last_insert_id() -> Unsigned<BigInt>; }

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

// mod default {
//     use store::FsStore;

//     use super::*;
//     use crate::{data::fs_store::FsStoreDb, ProblemID};
//     use std::collections::BTreeMap;

//     #[derive(Default, FsStore)]
//     pub struct Data {
//         #[meta]
//         data: BTreeMap<ProblemID, render_data::Statement>,
//     }
//     pub struct DefaultDB(FsStoreDb);
//     impl DefaultDB {
//         /// dir: base dir
//         pub fn new(dir: impl AsRef<std::path::Path>) -> Self {
//             Self(FsStoreDb::new(dir))
//             // std::fs::create_dir_all(dir.as_ref()).expect("create statement database dir");

//             // let dir = Handle::new(dir.as_ref());
//             // let data_file_path = dir.join("stmt.json");
//             // let data = if let Ok(file) = data_file_path.open_file() {
//             //     serde_json::from_reader(&file).unwrap_or_default()
//             // } else {
//             //     Default::default()
//             // };

//             // Self {
//             //     data: RwLock::new((data, data_file_path)),
//             //     dir,
//             // }
//         }
//     }
//     #[async_trait]
//     impl Manager for DefaultDB {
//         async fn get(&self, id: ProblemID) -> Result<Statement, DataError> {
//             self.0
//                 .table::<Data>()
//                 .read_transaction(|db| db.data.get(&id).map(From::from).ok_or(DataError::NotFound))
//         }
//         async fn insert(
//             &self,
//             id: ProblemID,
//             stmt: render_data::Statement,
//         ) -> Result<(), DataError> {
//             self.0.table::<Data>().write_transaction(|db| {
//                 db.data.insert(id, stmt);
//                 Ok(())
//             })
//         }
//         async fn get_assets(&self, id: ProblemID, name: &str) -> Result<NamedFile, DataError> {
//             let binding = self.0.table::<Data>().get_table()?;
//             let handle = binding.ctx.join(id.to_string()).join(name);
//             NamedFile::open(handle).map_err(DataError::IO)
//         }
//         async fn insert_assets(
//             &self,
//             id: ProblemID,
//             mut file: std::fs::File,
//             name: &str,
//         ) -> Result<(), DataError> {
//             let binding = self.0.table::<Data>().get_table()?;
//             let handle = binding.ctx.join(id.to_string()).join(name);
//             std::io::copy(&mut file, &mut handle.create_new_file()?).expect("copy file");
//             Ok(())
//         }
//         async fn get_metas(
//             &self,
//             max_count: u8,
//             offset: usize,
//             pattern: Option<String>,
//         ) -> Result<Vec<(ProblemID, StmtMeta)>, DataError> {
//             let re = if let Some(p) = pattern {
//                 regex::Regex::new(&p).ok()
//             } else {
//                 None
//             };

//             self.0.table::<Data>().read_transaction(|db| {
//                 let data = db
//                     .data
//                     .iter()
//                     .filter(|m| {
//                         re.as_ref()
//                             .map(|r| r.is_match(&m.1.meta.title))
//                             .unwrap_or(true)
//                     })
//                     .skip(offset)
//                     .take(max_count.into());

//                 Ok(data.map(|d| (*d.0, d.1.meta.clone())).collect())
//             })
//         }
//         async fn max_id(&self) -> Result<ProblemID, DataError> {
//             self.0
//                 .table::<Data>()
//                 .read_transaction(|db| Ok(db.data.iter().next_back().map(|v| *v.0).unwrap_or(0)))
//         }
//     }
// }

// pub use default::DefaultDB;
