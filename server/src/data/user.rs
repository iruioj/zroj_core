//! 用户数据库

use super::error::DataError;
use super::mysql::last_insert_id;
use super::mysql::schema::users;
use super::mysql::schema_model::User;
use super::types::*;
use crate::data::mysql::MysqlDb;
use crate::Override;
use crate::UserID;
use diesel::{self, prelude::*, Insertable};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;

pub type Mysql = DbManager;
pub type UserDB = dyn Manager + Sync + Send;

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
// #[async_trait(?Send)]
pub trait Manager {
    fn query_by_username(&self, username: &Username) -> Result<User, DataError>;
    fn query_by_userid(&self, uid: UserID) -> Result<User, DataError>;
    fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<UserID, DataError>;
    fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), DataError>;
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    username: &'a Username,
    password_hash: &'a str,
    email: &'a EmailAddress,
    register_time: &'a DateTime,
    gender: &'a Gender,
    name: &'a str,
    motto: &'a str,
}
pub struct DbManager(MysqlDb);

/// 数据库存储
impl DbManager {
    pub fn new(db: &MysqlDb) -> Self {
        Self(db.clone())
    }
}
impl Manager for DbManager {
    fn query_by_username(&self, username: &Username) -> Result<User, DataError> {
        self.0.transaction(|conn| {
            Ok(users::table
                .filter(users::username.eq(username))
                .first(conn)?)
        })
    }
    fn query_by_userid(&self, uid: UserID) -> Result<User, DataError> {
        self.0
            .transaction(|conn| Ok(users::table.filter(users::id.eq(uid)).first(conn)?))
    }
    fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<UserID, DataError> {
        self.0.transaction(|conn| {
            let new_user = NewUser {
                username,
                password_hash,
                email,
                register_time: &DateTime::now(),
                gender: &Gender::Private,
                name: "",
                motto: "",
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)?;
            let id: u64 = diesel::select(last_insert_id()).first(conn)?;
            Ok(id as UserID)
        })
    }
    fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            let mut user = users::table.filter(users::id.eq(uid)).first(conn)?;
            info.over(&mut user);
            diesel::update(users::table.filter(users::id.eq(uid)))
                .set(user)
                .execute(conn)?;
            Ok(())
        })
    }
}
