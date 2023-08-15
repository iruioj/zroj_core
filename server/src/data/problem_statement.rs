use super::error::Error;
use crate::ProblemID;
use actix_files::NamedFile;
use async_trait::async_trait;
use problem::render_data::{self, statement::StmtMeta};
use serde::Serialize;
use serde_ts_typing::TsType;

pub type StmtDB = dyn Manager + Sync + Send;

#[derive(Debug, Serialize, TsType)]
pub struct Statement {
    statement: problem::Mdast,
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
    async fn get(&self, id: ProblemID) -> Result<Option<Statement>, Error>;
    /// parse statement for reader and insert (update) it
    async fn insert(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), Error>;
    async fn get_assets(&self, id: ProblemID, name: &str) -> Option<NamedFile>;
    async fn insert_assets(
        &self,
        id: ProblemID,
        file: std::fs::File,
        name: &str,
    ) -> Result<(), Error>;
    /// get problem meta list, often used for problem listing
    async fn get_metas(
        &self,
        max_count: u8,
        pattern: Option<String>,
        min_id: Option<ProblemID>,
        max_id: Option<ProblemID>,
    ) -> Result<Vec<(ProblemID, StmtMeta)>, Error>;
    /// 当前的最大的题目 id
    async fn max_id(&self) -> Result<ProblemID, Error>;
}

mod default {
    use store::Handle;

    use super::*;
    use crate::ProblemID;
    use std::{collections::BTreeMap, io::Write, sync::RwLock};

    pub struct DefaultDB {
        data: RwLock<(BTreeMap<ProblemID, render_data::Statement>, std::fs::File)>,
        dir: Handle,
    }
    impl DefaultDB {
        pub fn new(dir: impl AsRef<std::path::Path>) -> Self {
            std::fs::create_dir_all(dir.as_ref()).expect("create statement database dir");
            let data_file = std::fs::File::options()
                .create(true)
                .write(true)
                .read(true)
                .open(dir.as_ref().join("stmt.json"))
                .expect("create/read data file");
            let data = serde_json::from_reader(&data_file).unwrap_or_default();
            Self {
                data: RwLock::new((data, data_file)),
                dir: Handle::new(dir.as_ref().to_path_buf()),
            }
        }
    }
    #[async_trait]
    impl Manager for DefaultDB {
        async fn get(&self, id: ProblemID) -> Result<Option<Statement>, Error> {
            Ok(self.data.read()?.0.get(&id).map(From::from))
        }
        async fn insert(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), Error> {
            let mut data = self.data.write()?;
            data.0.insert(id, stmt);
            data.1.set_len(0).expect("truncate data file");
            let r = serde_json::to_string(&data.0).map_err(Error::SerdeJson)?;
            data.1
                .write_all(r.as_bytes())
                .expect("fail to write problem statement data file");
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
        ) -> Result<(), Error> {
            let handle = self.dir.join(id.to_string()).join(name);
            std::io::copy(
                &mut file,
                &mut handle.create_new_file().map_err(Error::Store)?,
            )
            .expect("copy file");
            Ok(())
        }
        async fn get_metas(
            &self,
            max_count: u8,
            pattern: Option<String>,
            min_id: Option<ProblemID>,
            max_id: Option<ProblemID>,
        ) -> Result<Vec<(ProblemID, StmtMeta)>, Error> {
            let re = if let Some(p) = pattern {
                Some(regex::Regex::new(&p).map_err(Error::Regex)?)
            } else {
                None
            };

            let db = self.data.read()?;
            let data =
                db.0.iter()
                    .filter(|m| {
                        *m.0 >= min_id.unwrap_or_default()
                            && *m.0 <= max_id.unwrap_or(ProblemID::MAX)
                            && re
                                .as_ref()
                                .map(|r| r.is_match(&m.1.meta.title))
                                .unwrap_or(true)
                    })
                    .take(max_count.into());

            Ok(data.map(|d| (*d.0, d.1.meta.clone())).collect())
        }
        async fn max_id(&self) -> Result<ProblemID, Error> {
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
