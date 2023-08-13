use super::error::Error;
use crate::ProblemID;
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
    use super::*;
    use crate::ProblemID;
    use std::{collections::BTreeMap, path::PathBuf, sync::RwLock};

    pub struct DefaultDB {
        data: RwLock<BTreeMap<ProblemID, render_data::Statement>>,
        path: PathBuf,
    }
    impl DefaultDB {
        pub fn new(path: impl AsRef<std::path::Path>) -> Self {
            let data = if let Ok(file) = std::fs::File::open(path.as_ref()) {
                serde_json::from_reader(file).unwrap_or_default()
            } else {
                Default::default()
            };
            Self {
                data: RwLock::new(data),
                path: path.as_ref().to_path_buf(),
            }
        }
    }
    #[async_trait]
    impl Manager for DefaultDB {
        async fn get(&self, id: ProblemID) -> Result<Option<Statement>, Error> {
            Ok(self.data.read()?.get(&id).map(From::from))
        }
        async fn insert(&self, id: ProblemID, stmt: render_data::Statement) -> Result<(), Error> {
            self.data.write()?.insert(id, stmt);
            std::fs::write(
                &self.path,
                serde_json::to_string(
                    &self.data.read()? as &BTreeMap<ProblemID, render_data::Statement>
                )
                .map_err(Error::SerdeJson)?,
            )
            .expect("fail to write problem statement data file");
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
            let data = db
                .iter()
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
                .iter()
                .next_back()
                .map(|v| *v.0)
                .unwrap_or(0))
        }
    }
}

pub use default::DefaultDB;
