use super::error::DataError;
use crate::ProblemID;
use actix_files::NamedFile;
use async_trait::async_trait;
use problem::render_data::{self, statement::StmtMeta};
use serde::Serialize;
use serde_ts_typing::TsType;

pub type StmtDB = dyn Manager + Sync + Send;

#[derive(Debug, Serialize, TsType)]
pub struct Statement {
    statement: problem::render_data::Mdast,
    meta: StmtMeta,
}

impl From<&render_data::Statement> for Statement {
    fn from(value: &render_data::Statement) -> Self {
        Self {
            statement: value.statement.render_mdast(),
            meta: value.meta.clone(),
        }
    }
}

#[async_trait]
pub trait Manager {
    /// HTML statement
    async fn get(&self, id: ProblemID) -> Result<Statement, DataError>;
    /// parse statement for reader and insert (update) it
    async fn insert(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), DataError>;
    async fn get_assets(&self, id: ProblemID, name: &str) -> Option<NamedFile>;
    async fn insert_assets(
        &self,
        id: ProblemID,
        file: std::fs::File,
        name: &str,
    ) -> Result<(), DataError>;
    /// get problem meta list, often used for problem listing
    async fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<(ProblemID, StmtMeta)>, DataError>;
    /// 当前的最大的题目 id
    async fn max_id(&self) -> Result<ProblemID, DataError>;
}

mod default {
    use store::Handle;

    use super::*;
    use crate::ProblemID;
    use std::{collections::BTreeMap, sync::RwLock};

    pub struct DefaultDB {
        data: RwLock<(BTreeMap<ProblemID, render_data::Statement>, Handle)>,
        dir: Handle,
    }
    impl DefaultDB {
        /// dir: base dir
        pub fn new(dir: impl AsRef<std::path::Path>) -> Self {
            std::fs::create_dir_all(dir.as_ref()).expect("create statement database dir");

            let dir = Handle::new(dir.as_ref());
            let data_file_path = dir.join("stmt.json");
            let data = if let Ok(file) = data_file_path.open_file() {
                serde_json::from_reader(&file).unwrap_or_default()
            } else {
                Default::default()
            };

            Self {
                data: RwLock::new((data, data_file_path)),
                dir,
            }
        }
    }
    #[async_trait]
    impl Manager for DefaultDB {
        async fn get(&self, id: ProblemID) -> Result<Statement, DataError> {
            self.data
                .read()?
                .0
                .get(&id)
                .map(From::from)
                .ok_or(DataError::NotFound)
        }
        async fn insert(
            &self,
            id: ProblemID,
            stmt: render_data::Statement,
        ) -> Result<(), DataError> {
            let mut data = self.data.write()?;
            data.0.insert(id, stmt);
            let r = serde_json::to_string(&data.0)?;
            std::fs::write(data.1.path(), r).expect("write data to file");
            drop(data);
            Ok(())
        }
        async fn get_assets(&self, id: ProblemID, name: &str) -> Option<NamedFile> {
            let handle = self.dir.join(id.to_string()).join(name);
            NamedFile::open(handle).ok()
        }
        async fn insert_assets(
            &self,
            id: ProblemID,
            mut file: std::fs::File,
            name: &str,
        ) -> Result<(), DataError> {
            let handle = self.dir.join(id.to_string()).join(name);
            std::io::copy(&mut file, &mut handle.create_new_file()?).expect("copy file");
            Ok(())
        }
        async fn get_metas(
            &self,
            max_count: u8,
            offset: usize,
            pattern: Option<String>,
        ) -> Result<Vec<(ProblemID, StmtMeta)>, DataError> {
            let re = if let Some(p) = pattern {
                regex::Regex::new(&p).ok()
            } else {
                None
            };

            let db = self.data.read()?;
            let data =
                db.0.iter()
                    .filter(|m| {
                        re.as_ref()
                            .map(|r| r.is_match(&m.1.meta.title))
                            .unwrap_or(true)
                    })
                    .skip(offset)
                    .take(max_count.into());

            Ok(data.map(|d| (*d.0, d.1.meta.clone())).collect())
        }
        async fn max_id(&self) -> Result<ProblemID, DataError> {
            Ok(self
                .data
                .read()?
                .0
                .iter()
                .next_back()
                .map(|v| *v.0)
                .unwrap_or(0))
        }
    }
}

pub use default::DefaultDB;
