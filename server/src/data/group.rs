use super::super::data::error::Result;
use super::schema::Group;
use crate::{auth::UserID, problem::GroupID};
use async_trait::async_trait;
use std::sync::Arc;
pub type AManager = dyn Manager + Sync + Send;

#[async_trait]
pub trait Manager {
    /// insert a group with a name which only contains the owner
    /// returns None if this group name is already taken
    /// otherwise returns group id
    async fn new_group(&self, owner: UserID, name: String) -> Result<Option<GroupID>>;
    async fn group_contains(&self, id: GroupID, uid: i32) -> Result<bool>;
    /// returns false if uid already exists
    async fn group_insert(&self, id: GroupID, uid: UserID) -> Result<bool>;
    /// return false if uid does not exist
    async fn group_delete(&self, id: GroupID, uid: UserID) -> Result<bool>;
    async fn get_groupid(&self, name: &String) -> Result<Option<GroupID>>;
    async fn get_group_users(&self, uid: GroupID) -> Result<Option<Vec<UserID>>>;
    fn to_amanager(self) -> Arc<AManager>;
}

pub use hashmap::FsManager;
mod hashmap {
    use std::{collections::HashMap, path::PathBuf, sync::RwLock};

    use serde::{Deserialize, Serialize};
    use serde_json::from_str;

    use super::*;

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
            let mut r = Self::load(&path).unwrap_or(Data(HashMap::new(), Vec::new()));
            if r.1.len() == 0 {
                let g = Group {
                    name: "public".to_string(),
                    id: 0,
                    owner: 0,
                    users: Vec::new(),
                };
                r.0.insert(g.name.clone(), g.id);
                r.1.push(g);
            }
            Self {
                data: RwLock::new(r),
                path: path.clone(),
            }
        }
        fn load(path: &PathBuf) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("Fail to read from path: {}", path.display()))?;
            Ok(from_str::<Data>(&s)
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
        async fn new_group(&self, owner: UserID, name: String) -> Result<Option<GroupID>> {
            let mut guard = self.data.write()?;
            let id = guard.1.len() as GroupID;
            let g = Group {
                name,
                id,
                owner,
                users: vec![id],
            };
            if let Some(_) = guard.0.insert(g.name.clone(), g.id) {
                return Ok(None);
            }
            guard.1[id as usize] = g;
            drop(guard);
            self.save();
            Ok(Some(id))
        }
        async fn group_contains(&self, id: GroupID, uid: UserID) -> Result<bool> {
            let guard = self.data.read()?;
            Ok(match guard.1[id as usize].users.binary_search(&uid) {
                Ok(_) => true,
                Err(_) => false,
            })
        }
        async fn group_insert(&self, id: GroupID, uid: UserID) -> Result<bool> {
            let mut guard = self.data.write()?;
            let vec = &mut guard.1[id as usize].users;
            let index = match vec.binary_search(&uid) {
                Ok(_) => return Ok(false),
                Err(index) => index,
            };
            vec.insert(index, uid);
            self.save();
            Ok(true)
        }
        async fn group_delete(&self, id: GroupID, uid: UserID) -> Result<bool> {
            let mut guard = self.data.write()?;
            let vec = &mut guard.1[id as usize].users;
            let index = match vec.binary_search(&uid) {
                Ok(index) => index,
                Err(_) => return Ok(false),
            };
            vec.remove(index);
            self.save();
            Ok(true)
        }
        async fn get_groupid(&self, name: &String) -> Result<Option<GroupID>> {
            let guard = self.data.read()?;
            Ok(match guard.0.get(name) {
                Some(&uid) => Some(uid),
                None => None,
            })
        }
        async fn get_group_users(&self, uid: GroupID) -> Result<Option<Vec<UserID>>> {
            let guard = self.data.read()?;
            if uid < 0 || uid as usize > guard.1.len() {
                return Ok(None);
            }
            Ok(Some(guard.1[uid as usize].users.clone()))
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
