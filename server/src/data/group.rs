use super::super::data::error::Error;
use crate::{GroupID, UserID};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AManager = dyn Manager + Sync + Send;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    /// 用户组 ID
    pub id: GroupID,
    /// 用户组名称
    pub name: String,
    /// 用户
    pub users: Vec<UserID>,
}

#[async_trait]
pub trait Manager {
    async fn new_group(&self, name: String) -> Result<GroupID>;
    async fn contains(&self, gid: GroupID, uid: UserID) -> Result<bool>;
    /// 返回添加的用户数
    async fn insert(&self, gid: GroupID, users: &[UserID]) -> Result<usize>;
    /// return false if delete_uid does not exist
    async fn delete(&self, gid: GroupID, delete_uid: UserID) -> Result<bool>;
    async fn get_info(&self, gid: GroupID) -> Result<Option<Group>>;
    fn to_amanager(self) -> Arc<AManager>;
}

pub use hashmap::FsManager;
mod hashmap {
    use super::*;
    use std::{collections::HashMap, path::PathBuf, sync::RwLock};

    #[derive(Serialize, Deserialize)]
    struct Data(HashMap<String, GroupID>, Vec<Group>);

    /// 文件系统存储信息
    #[derive(Serialize, Deserialize)]
    pub struct FsManager {
        data: RwLock<Data>,
        path: PathBuf,
    }

    impl FsManager {
        pub fn new(path: PathBuf) -> Self {
            let r = Self::load(&path).unwrap_or(Data(HashMap::new(), Vec::new()));
            Self {
                data: RwLock::new(r),
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
            let data = self.data.read().expect("Fail to fetch data when saving");
            let s = serde_json::to_string::<Data>(&data).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s).unwrap_or_else(|_| {
                panic!("Fail to write user data to path: {}", self.path.display())
            });
        }
    }
    #[async_trait]
    impl super::Manager for FsManager {
        async fn new_group(&self, name: String) -> Result<GroupID> {
            let mut data = self.data.write()?;
            let id = data.1.len() as GroupID;
            let g = Group {
                name: name.clone(),
                id,
                users: Vec::new(),
            };
            if data.0.contains_key(&name) {
                return Err(Error::DuplicatedGroupName(name));
            }
            data.0.insert(name, g.id);
            data.1.push(g);
            drop(data);
            self.save();
            Ok(id)
        }
        async fn contains(&self, gid: GroupID, uid: UserID) -> Result<bool> {
            let data = self.data.read()?;
            Ok(data.1[gid as usize].users.contains(&uid))
        }
        async fn insert(&self, gid: GroupID, users: &[UserID]) -> Result<usize> {
            let mut data = self.data.write()?;
            let v = &mut data.1[gid as usize].users;
            let count = users
                .iter()
                .filter(|u| {
                    if !v.contains(u) {
                        v.push(**u);
                        true
                    } else {
                        false
                    }
                })
                .count();
            if count > 0 {
                drop(data);
                self.save();
            }
            Ok(count)
        }
        async fn delete(&self, gid: GroupID, delete_uid: UserID) -> Result<bool> {
            let mut data = self.data.write()?;
            let v = &mut data.1[gid as usize];
            if let Some(index) = v.users.iter().position(|c| c == &delete_uid) {
                v.users.remove(index);
                self.save();
                Ok(true)
            } else {
                Ok(false)
            }
        }
        async fn get_info(&self, id: GroupID) -> Result<Option<Group>> {
            let data = self.data.read()?;
            if id as usize > data.1.len() {
                return Ok(None);
            }
            Ok(Some(data.1[id as usize].clone()))
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
