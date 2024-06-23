//! 用户数据库

use super::super::{
    error::DataError,
    mysql::{last_insert_id, schema::users, schema_model::User, MysqlDb},
    types::*,
};
use crate::UserID;
use diesel::{self, prelude::*, Insertable};

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

pub struct UserDB(MysqlDb);

impl UserDB {
    pub fn new(db: &MysqlDb) -> Self {
        Self(db.clone())
    }
    pub fn query_by_username(&self, username: &Username) -> Result<User, DataError> {
        self.0.transaction(|conn| {
            Ok(users::table
                .filter(users::username.eq(username))
                .first(conn)?)
        })
    }
    pub fn query_by_userid(&self, uid: UserID) -> Result<User, DataError> {
        self.0
            .transaction(|conn| Ok(users::table.filter(users::id.eq(uid)).first(conn)?))
    }
    pub fn new_user(
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
    pub fn update(
        &self,
        uid: UserID,
        password_hash: Option<String>,
        email: Option<EmailAddress>,
        motto: Option<String>,
        name: Option<String>,
        gender: Option<Gender>,
    ) -> Result<(), DataError> {
        self.0.transaction(|conn| {
            let mut user: User = users::table.filter(users::id.eq(uid)).first(conn)?;
            if let Some(pw) = password_hash {
                user.password_hash = pw;
            }
            if let Some(e) = email {
                user.email = e;
            }
            if let Some(m) = motto {
                user.motto = m;
            }
            if let Some(n) = name {
                user.name = n;
            }
            if let Some(g) = gender {
                user.gender = g;
            }
            diesel::update(users::table.filter(users::id.eq(uid)))
                .set(user)
                .execute(conn)?;
            Ok(())
        })
    }
}
