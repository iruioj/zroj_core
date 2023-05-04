use super::super::data::error::{Error, Result};
use super::schema::Group;
use crate::{auth::UserID, problem::GroupID};
use async_trait::async_trait;
use std::sync::Arc;
pub type AManager = dyn Manager + Sync + Send;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupUsers(Vec<UserID>);
impl GroupUsers {
    pub fn new(id: UserID) -> Self {
        Self { 0: vec![id] }
    }
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Group users not maintained properly")
    }
    pub fn from_str(s: &String) -> Self {
        serde_json::from_str(s).expect("Group users not maintained properly")
    }
    pub fn contains(&self, uid: UserID) -> bool {
        match self.0.binary_search(&uid) {
            Ok(_) => true,
            _ => false,
        }
    }
    pub fn insert(&mut self, uid: UserID) -> bool {
        let index = match self.0.binary_search(&uid) {
            Ok(_) => return false,
            Err(index) => index,
        };
        self.0.insert(index, uid);
        true
    }
    pub fn delete(&mut self, uid: UserID) -> bool {
        let index = match self.0.binary_search(&uid) {
            Ok(index) => index,
            Err(_) => return false,
        };
        self.0.remove(index);
        true
    }
}

#[async_trait]
pub trait Manager {
    /// insert a group with a name which only contains the owner
    /// returns None if this group name is already taken
    /// otherwise returns group id
    async fn new_group(&self, owner: UserID, name: String) -> Result<Option<GroupID>>;
    async fn group_contains(&self, gid: GroupID, uid: i32) -> Result<bool>;
    /// returns false if uid already exists
    async fn group_insert(&self, uid: UserID, gid: GroupID, users: &Vec<UserID>) -> Result<usize>;
    /// return false if uid does not exist
    async fn group_delete(&self, uid: UserID, gid: GroupID, delete_uid: UserID) -> Result<bool>;
    async fn get_groupid(&self, name: &String) -> Result<Option<GroupID>>;
    async fn get_group_info(&self, id: GroupID) -> Result<Option<Group>>;
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
            let mut r = Self::load(&path).unwrap_or(Data(HashMap::new(), Vec::new()));
            if r.1.len() == 0 {
                let g = Group {
                    name: "public".to_string(),
                    id: 0,
                    owner: 0,
                    users: GroupUsers::new(0),
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
        async fn new_group(&self, owner: UserID, name: String) -> Result<Option<GroupID>> {
            let mut guard = self.data.write()?;
            let id = guard.1.len() as GroupID;
            let g = Group {
                name,
                id,
                owner,
                users: GroupUsers::new(owner),
            };
            if let Some(_) = guard.0.insert(g.name.clone(), g.id) {
                return Ok(None);
            }
            guard.1[id as usize] = g;
            drop(guard);
            self.save();
            Ok(Some(id))
        }
        async fn group_contains(&self, gid: GroupID, uid: UserID) -> Result<bool> {
            if gid == 0 {
                return Ok(true);
            }
            let guard = self.data.read()?;
            Ok(guard.1[gid as usize].users.contains(uid))
        }
        async fn group_insert(
            &self,
            uid: UserID,
            gid: GroupID,
            users: &Vec<UserID>,
        ) -> Result<usize> {
            if gid == 0 {
                return Err(Error::Forbidden(
                    "Group 0 is not modifyable".to_string(),
                ));
            }
            let mut guard = self.data.write()?;
            let v = &mut guard.1[gid as usize];
            if v.owner != uid {
                return Err(Error::Forbidden(
                    "Only group owner can perform insert operation".to_string(),
                ));
            }
            let v = &mut v.users;
            let mut count: usize = 0;
            for i in users {
                if v.insert(i.clone()) {
                    count += 1;
                }
            }
            if count > 0 {
                self.save();
            }
            Ok(count)
        }
        async fn group_delete(&self, uid: UserID, gid: GroupID, delete_uid: UserID) -> Result<bool> {
            if gid == 0 {
                return Err(Error::Forbidden(
                    "Group 0 is not modifyable".to_string(),
                ));
            }
            let mut guard = self.data.write()?;
            let v = &mut guard.1[gid as usize];
            if v.owner != uid {
                return Err(Error::Forbidden(
                    "Only group owner can perform delete operation".to_string(),
                ));
            }
            let result = v.users.delete(delete_uid);
            if result {
                self.save();
            }
            Ok(true)
        }
        async fn get_groupid(&self, name: &String) -> Result<Option<GroupID>> {
            let guard = self.data.read()?;
            Ok(match guard.0.get(name) {
                Some(&uid) => Some(uid),
                None => None,
            })
        }
        async fn get_group_info(&self, id: GroupID) -> Result<Option<Group>> {
            let guard = self.data.read()?;
            if id < 0 || id as usize > guard.1.len() {
                return Ok(None);
            }
            Ok(Some(guard.1[id as usize].clone()))
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
