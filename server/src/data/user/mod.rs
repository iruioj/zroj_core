//! 用户数据库

use super::error::DataError;
use super::types::*;
use crate::{Override, UserID};
use async_trait::async_trait;
#[cfg(feature = "mysql")]
use diesel::*;
use serde::{Deserialize, Serialize};

pub type UserDB = dyn Manager + Sync + Send;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "mysql", derive(Queryable, AsChangeset))]
#[cfg_attr(feature = "mysql", diesel(table_name = database::users))]
pub struct User {
    /// 用户 id
    pub id: UserID,
    /// 用户名
    pub username: Username,
    /// 密码的 hash 值
    pub password_hash: String,
    /// 真实姓名
    pub name: String,
    /// 邮箱
    pub email: EmailAddress,
    /// 格言
    pub motto: String,
    /// 注册时间
    pub register_time: DateTime,
    /// 性别
    pub gender: Gender,
}

#[derive(Serialize, TsType)]
pub struct UserDisplayInfo {
    id: u32,
    username: Username,
    email: EmailAddress,
    motto: String,
    name: String,
    register_time: String,
    gender: Gender,
}
impl From<User> for UserDisplayInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            motto: value.motto,
            name: value.name,
            register_time: value.register_time.to_string(),
            gender: value.gender,
        }
    }
}

#[derive(Serialize, TsType)]
pub struct UserEditInfo {
    id: u32,
    username: String,
    email: String,
    motto: String,
    name: String,
    register_time: String,
    gender: Gender,
}

impl From<User> for UserEditInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username.to_string(),
            email: value.email.to_string(),
            motto: value.motto,
            name: value.name,
            register_time: value.register_time.to_string(),
            gender: value.gender,
        }
    }
}

#[derive(Deserialize, TsType)]
pub struct UserUpdateInfo {
    password_hash: Option<String>,
    email: Option<EmailAddress>,
    motto: Option<String>,
    name: Option<String>,
    gender: Option<Gender>,
}

impl crate::Override<User> for UserUpdateInfo {
    fn over(self, origin: &mut User) {
        if let Some(pw) = self.password_hash {
            origin.password_hash = pw;
        }
        if let Some(e) = self.email {
            origin.email = e;
        }
        if let Some(m) = self.motto {
            origin.motto = m;
        }
        if let Some(n) = self.name {
            origin.name = n;
        }
        if let Some(g) = self.gender {
            origin.gender = g;
        }
    }
}

// Result<Option<...>> pattern: Err 表示出错， None 表示未查到，Some 表示查到的值
#[async_trait(?Send)]
pub trait Manager {
    async fn query_by_username(&self, username: &Username) -> Result<User, DataError>;
    async fn query_by_userid(&self, uid: UserID) -> Result<User, DataError>;
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, DataError>;
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), DataError>;
}

#[cfg(feature = "mysql")]
pub use database::DbManager;

#[cfg(feature = "mysql")]
mod database;

pub use default::DefaultDB;
use serde_ts_typing::TsType;

mod default;
