//! manages data of a problem that are related to website but not problem itself
//! for example, problem access and create date

pub type AManager = dyn Manager + Sync + Send;
use super::error::{Error, Result};
use super::schema::ProblemAccess;
use crate::data::group::AManager as GroupAManager;
use crate::{data::schema::ProblemConfig, ProblemID, UserID};
use async_trait::async_trait;
pub use hashmap::FsManager;
use std::sync::Arc;

#[async_trait]
pub trait Manager {
    /// get config value, must provide userid to check access
    async fn get(&self, pid: ProblemID, uid: UserID) -> Result<ProblemConfig>;
    /// insert config for a new problem
    async fn insert(&mut self, pid: ProblemID, value: ProblemConfig) -> Result<()>;
    /// update the config of an existing problem, must provide userid to check access
    async fn set(&mut self, pid: ProblemID, uid: UserID, value: ProblemConfig) -> Result<()>;
    /// query the access of a user for a problem
    async fn get_access(&self, pid: ProblemID, uid: UserID) -> Result<ProblemAccess>;
    fn to_amanager(self) -> Arc<AManager>;
}

mod hashmap {

    use crate::data::schema::UserGroup;

    use super::*;
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, path::PathBuf, sync::RwLock};

    #[derive(Serialize, Deserialize)]
    struct Data(HashMap<ProblemID, ProblemConfig>);

    /// 文件系统存储信息
    pub struct FsManager {
        data: RwLock<Data>,
        groups: Arc<GroupAManager>,
        path: PathBuf,
    }

    impl FsManager {
        pub fn new(path: PathBuf, groups: Arc<GroupAManager>) -> Self {
            let r = Self::load(&path).unwrap_or(Data(HashMap::new()));
            Self {
                data: RwLock::new(r),
                groups,
                path,
            }
        }
        fn load(path: &PathBuf) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("Fail to read from path: {}", path.display()))?;
            serde_json::from_str::<Data>(&s)
                .map_err(|_| eprintln!("Fail to parse file content as user data"))
        }
        /// save data to json file, must be saved or panic!!!
        fn save(&self) {
            let guard = self.data.read().expect("Fail to fetch guard when saving");
            let s = serde_json::to_string::<Data>(&guard).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s).unwrap_or_else(|_| panic!("Fail to write user data to path: {}",
                self.path.display()));
        }
        fn _get(&self, pid: ProblemID) -> Result<ProblemConfig> {
            let guard = self.data.read()?;
            Ok(guard
                .0
                .get(&pid)
                .ok_or(Error::InvalidArgument(format!(
                    "Problem {} does not exist",
                    pid
                )))?
                .clone())
        }
    }
    #[async_trait]
    impl super::Manager for FsManager {
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
        async fn get(&self, pid: ProblemID, uid: UserID) -> Result<ProblemConfig> {
            let value = self._get(pid)?;
            if value.owner != uid {
                Err(Error::Forbidden("Only owner can access config".to_string()))
            } else {
                Ok(value)
            }
        }
        async fn insert(&mut self, pid: ProblemID, value: ProblemConfig) -> Result<()> {
            let mut guard = self.data.write()?;
            if guard.0.get(&pid).is_some() {
                Err(Error::InvalidArgument(format!("Problem {} already exists", pid)))
            } else {
                if guard.0.insert(pid, value).is_some() {
                    panic!("impossible")
                }
                self.save();
                Ok(())
                /*if (*v).owner != uid {
                    Err(Error::Forbidden(("only owner can access the config".to_string())))
                } else {
                    *v = value;
                    self.save();
                    Ok(())
                }*/
            }
        }
        async fn set(&mut self, pid: ProblemID, uid: UserID, value: ProblemConfig) -> Result<()> {
            let mut guard = self.data.write()?;
            if let Some(v) = guard.0.get_mut(&pid) {
                if v.owner != uid {
                    Err(Error::Forbidden("only owner can access the config".to_string()))
                } else {
                    *v = value;
                    self.save();
                    Ok(())
                }
            } else {
                Err(Error::InvalidArgument(format!("No such problem: {}", pid)))
            }
        }
        async fn get_access(&self, pid: ProblemID, uid: UserID) -> Result<ProblemAccess> {
            let mut t: ProblemAccess = ProblemAccess::None;
            let value = self._get(pid)?;
            for (ug, a) in value.access {
                let flag = match ug {
                    UserGroup::User(id) => id == uid,
                    UserGroup::Group(gid) => self.groups.group_contains(gid, uid).await?,
                };
                if flag && a > t {
                    t = a
                }
            }
            Ok(t)
        }
    }
}
