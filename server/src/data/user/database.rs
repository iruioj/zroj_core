use crate::data::error::DataError;
use crate::data::mysql::MysqlDb;
use crate::data::user::{async_trait, Manager, User, UserID, UserUpdateInfo};
use crate::data::{notfound_as_none, types::*};
use crate::Override;
use diesel::{self, prelude::*, table, Insertable};

// 必须保证和 User 的字段顺序相同， 不然 query 会出问题
table! {
    users (id) {
        /// id should be auto increment
        id -> Unsigned<Integer>,
        username -> Text,
        password_hash -> Text,
        name -> Text,
        email -> Text,
        motto -> Text,
        register_time -> BigInt,
        gender -> Text,
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a Username,
    pub password_hash: &'a str,
    pub email: &'a EmailAddress,
    pub register_time: &'a DateTime,
    pub gender: &'a Gender,
}
pub struct DbManager(MysqlDb);

/// 数据库存储
impl DbManager {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self(MysqlDb::new(url))
    }
}
#[async_trait(?Send)]
impl Manager for DbManager {
    async fn query_by_username(&self, username: &Username) -> Result<Option<User>, DataError> {
        notfound_as_none(self.0.transaction(|conn| {
            users::table
                .filter(users::username.eq(username))
                .first(conn)
                .map_err(DataError::Diesel)
        }))
    }
    async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>, DataError> {
        notfound_as_none(self.0.transaction(|conn| {
            users::table
                .filter(users::id.eq(uid))
                .first(conn)
                .map_err(DataError::Diesel)
        }))
    }
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, DataError> {
        self.0.transaction(|conn| {
            let new_user = NewUser {
                username,
                password_hash,
                email,
                register_time: &DateTime::now(),
                gender: &JsonStr(GenderInner::Private),
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)?;
            Ok(users::table.order(users::id.desc()).first::<User>(conn)?)
        })
    }
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            let mut user = users::table
                .filter(users::id.eq(uid))
                .first(conn)
                .map_err(DataError::Diesel)?;
            info.over(&mut user);
            diesel::update(users::table.filter(users::id.eq(uid)))
                .set(user)
                .execute(conn)?;
            Ok(())
        })
    }
}
