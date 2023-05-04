//! manages data of a problem that are related to website but not problem itself
//! for example, problem access and create date

pub type AManager = dyn Manager + Sync + Send;
use async_trait::async_trait;
pub use hashmap::FsManager;
use std::sync::Arc;
use crate::{ProblemID, data::schema::ProblemConfig};

#[async_trait]
pub trait Manager {
    fn to_amanager(self) -> Arc<AManager>;
}

mod hashmap {
    use serde::{Serialize, Deserialize};

    use super::*;
    use std::{collections::HashMap, path::PathBuf, sync::RwLock};

    #[derive(Serialize, Deserialize)]
    struct Data(HashMap<ProblemID, ProblemConfig>);

    /// 文件系统存储信息
    #[derive(Serialize, Deserialize)]
    pub struct FsManager {
        data: RwLock<Data>,
        path: PathBuf,
    }

    impl FsManager {
        pub fn new(path: PathBuf) -> Self {
            let mut r = Self::load(&path).unwrap_or(Data(HashMap::new()));
            Self {
                data: RwLock::new(r),
                path: path.clone(),
            }
        }
        fn load(path: &PathBuf) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("Fail to read from path: {}", path.display()))?;
            Ok(serde_json::from_str::<Data>(&s)
                .map_err(|_| eprintln!("Fail to parse file content as user data"))?)
        }
        /// save data to json file, must be saved or panic!!!
        fn save(&self) {
            let guard = self.data.read().expect("Fail to fetch guard when saving");
            let s = serde_json::to_string::<Data>(&guard).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s).expect(&format!(
                "Fail to write user data to path: {}",
                self.path.display()
            ));
        }
    }
    #[async_trait]
    impl super::Manager for FsManager {
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
