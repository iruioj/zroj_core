use super::error::Error;
use crate::ProblemID;
use async_trait::async_trait;
use problem::render_data;
use serde::Serialize;

pub type StmtDB = dyn Manager + Sync + Send;

#[derive(Debug, Serialize)]
pub struct Statement {
    statement: String,
    meta: render_data::statement::Meta,
}

#[async_trait]
pub trait Manager {
    /// HTML statement
    async fn get(&self, id: ProblemID) -> Result<Option<Statement>, Error>;
    /// parse statement for reader and insert (update) it
    async fn insert(&self, id: ProblemID, content: &[u8]) -> Result<(), Error>;
    /// most of the time for debugging
    async fn get_metas(&self) -> Result<Vec<(ProblemID, render_data::statement::Meta)>, Error>;
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
            Ok(self.data.read()?.get(&id).map(|s| Statement {
                statement: s.statement.render_html(),
                meta: s.meta.clone(),
            }))
        }
        async fn insert(
            &self,
            id: ProblemID,
            content: &[u8],
        ) -> Result<(), Error> {
            let s: render_data::Statement =
                serde_json::from_slice(content).map_err(Error::SerdeJson)?;
            self.data.write()?.insert(id, s);
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
        async fn get_metas(&self) -> Result<Vec<(ProblemID, render_data::statement::Meta)>, Error> {
            Ok(self
                .data
                .read()?
                .iter()
                .map(|d| (*d.0, d.1.meta.clone()))
                .collect())
        }
    }
}

pub use default::DefaultDB;