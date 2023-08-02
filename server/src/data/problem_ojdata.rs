//! 题目的评测数据

use super::error::Error;
use crate::ProblemID;
use async_trait::async_trait;
use problem::StandardProblem;

pub type OJDataDB = dyn Manager + Sync + Send;

#[async_trait]
pub trait Manager {
    /// HTML statement
    async fn get(&self, id: ProblemID) -> Result<StandardProblem, Error>;
    /// parse statement for reader and insert (update) it
    async fn insert(&self, id: ProblemID, data: StandardProblem) -> Result<(), Error>;
}

mod default {
    use super::*;
    use judger::Handle;
    use store::FsStore;

    pub struct DefaultDB {
        root: Handle,
    }
    impl DefaultDB {
        pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
            let path = path.as_ref();
            if path.exists() {
                if path.is_dir() {
                    Ok(Self {
                        root: Handle::new(path),
                    })
                } else {
                    panic!("not a dir {}", path.display())
                }
            } else {
                std::fs::create_dir_all(path).unwrap();
                Ok(Self {
                    root: Handle::new(path),
                })
            }
        }
    }
    #[async_trait]
    impl Manager for DefaultDB {
        async fn get(&self, id: ProblemID) -> Result<StandardProblem, Error> {
            let handle = self.root.join(id.to_string());
            Ok(StandardProblem::open(&handle).map_err(Error::Store)?)
        }
        async fn insert(&self, id: ProblemID, mut data: StandardProblem) -> Result<(), Error> {
            let handle = self.root.join(id.to_string());
            if handle.path().exists() { // 删掉以前的数据（危险的操作，可以考虑加入备份的机制）
                std::fs::remove_dir_all(handle.path()).unwrap();
            }
            data.save(&handle).map_err(Error::Store)
        }
    }
}
pub use default::DefaultDB;
