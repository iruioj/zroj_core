//! 默认实现方式（无 sql 依赖）

use super::*;
use crate::data::FsStoreLock;
use std::collections::HashMap;
use store::FsStore;

#[derive(Default, FsStore)]
struct Data {
    #[meta]
    name_map: HashMap<Username, UserID>,
    #[meta]
    users: Vec<User>,
}

/// 文件系统存储信息
pub struct DefaultDB(FsStoreLock<Data>);

impl DefaultDB {
    /// 将数据用 serde 暴力存储在 path 对应的文件中
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, DataError> {
        Ok(Self(FsStoreLock::try_load(path)?))
    }
}

#[async_trait(?Send)]
impl super::Manager for DefaultDB {
    async fn query_by_username(&self, username: &Username) -> Result<User, DataError> {
        let data = self.0.read()?;
        Ok(data.users[*data.name_map.get(username).ok_or(DataError::NotFound)? as usize].clone())
    }
    async fn query_by_userid(&self, uid: UserID) -> Result<User, DataError> {
        let data = self.0.read()?;
        data.users
            .get(uid as usize)
            .ok_or(DataError::NotFound)
            .cloned()
    }
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, DataError> {
        let mut data = self.0.write()?;
        let new_user = User {
            id: data.users.len() as UserID,
            username: username.clone(),
            password_hash: password_hash.to_string(),
            email: email.clone(),
            motto: String::new(),
            name: String::new(),
            register_time: DateTime::now(),
            gender: Gender::Private,
        };
        data.name_map.insert(username.clone(), new_user.id);
        data.users.push(new_user.clone());
        drop(data);
        Ok(new_user)
    }
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), DataError> {
        let mut data = self.0.write()?;
        let value = data
            .users
            .get_mut(uid as usize)
            .ok_or(DataError::NotFound)?;
        info.over(value);
        drop(data);
        Ok(())
    }
}
