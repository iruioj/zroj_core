//! 默认实现方式（无 sql 依赖）

use super::*;
use std::sync::RwLock;
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, Default)]
struct Data {
    name_map: HashMap<Username, UserID>,
    users: Vec<User>,
}

/// 文件系统存储信息
#[derive(Serialize, Deserialize)]
pub struct DefaultDB {
    data: RwLock<Data>,
    path: PathBuf,
}

impl DefaultDB {
    /// 将数据用 serde 暴力存储在 path 对应的文件中
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        let r = Self::load(path.as_ref()).unwrap_or_default();
        Self {
            data: RwLock::new(r),
            path: path.as_ref().into(),
        }
    }
    fn load(path: impl AsRef<std::path::Path>) -> std::result::Result<Data, ()> {
        let s = std::fs::read_to_string(path.as_ref()).map_err(|_| {
            eprintln!(
                "warning: fail to read from path: {}",
                path.as_ref().display()
            )
        })?;
        serde_json::from_str::<Data>(&s)
            .map_err(|_| eprintln!("warning: fail to parse file content as user data"))
    }
    /// save data to json file, must be saved or panic!!!
    fn save(&self) {
        let data = self.data.read().expect("Fail to fetch data when saving");
        let s = serde_json::to_string::<Data>(&data).expect("Fail to parse user data as json");
        std::fs::write(&self.path, s)
            .unwrap_or_else(|_| panic!("Fail to write user data to path: {}", self.path.display()));
    }
}

#[async_trait(?Send)]
impl super::Manager for DefaultDB {
    async fn query_by_username(&self, username: &Username) -> Result<Option<User>, Error> {
        let data = self.data.read()?;
        Ok(data
            .name_map
            .get(username)
            .map(|uid| data.users[*uid as usize].clone()))
    }
    async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>, Error> {
        let data = self.data.read()?;
        Ok(data.users.get(uid as usize).cloned())
    }
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, Error> {
        let mut data = self.data.write()?;
        let new_user = User {
            id: data.users.len() as UserID,
            username: username.clone(),
            password_hash: password_hash.to_string(),
            email: email.clone(),
            motto: String::new(),
            name: String::new(),
            register_time: DateTime::now(),
            gender: JsonStr(GenderInner::Private),
            // groups: serde_json::to_string(&Vec::<GroupID>::new()).unwrap(),
        };
        data.name_map.insert(username.clone(), new_user.id);
        data.users.push(new_user.clone());
        drop(data);
        self.save();
        Ok(new_user)
    }
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), Error> {
        let mut data = self.data.write()?;
        let value = data
            .users
            .get_mut(uid as usize)
            .ok_or(Error::InvalidArgument(format!(
                "userid = {} violates range",
                uid
            )))?;
        info.over(value);
        drop(data);
        self.save();
        Ok(())
    }
}
