//! 用户数据库

use super::error::Error;
use super::types::*;
use crate::app::user::UserUpdateInfo;
use crate::{Override, UserID};
use async_trait::async_trait;
#[cfg(feature = "mysql")]
use diesel::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AManager = dyn Manager + Sync + Send;

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

// Result<Option<...>> pattern: Err 表示出错， None 表示未查到，Some 表示查到的值
#[async_trait]
pub trait Manager {
    async fn query_by_username(&self, username: &Username) -> Result<Option<User>, Error>;
    async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>, Error>;
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, Error>;
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), Error>;
    fn to_amanager(self) -> Arc<AManager>;
}

#[cfg(feature = "mysql")]
pub use database::DbManager;

#[cfg(feature = "mysql")]
mod database;

pub use default::FsManager;

mod default;
