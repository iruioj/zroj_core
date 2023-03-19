//! imp struct for different database queries
use crate::schema::{users, User, NewUser};
use crate::{MysqlPool};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error;
use diesel::result::Error as DieselError;
use diesel::{
    self, prelude::*,
    r2d2::{ConnectionManager, Pool},
    mysql::{MysqlConnection}, 
};


#[derive(Debug)]
pub struct DbError(String);

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }  
}
impl std::convert::From <DieselError> for DbError {
    fn from(err: DieselError) -> Self {
        DbError(format!("InternalError occured while database query: {}", err.to_string()))
    }
}
impl std::convert::From <r2d2::Error> for DbError {
    fn from(err: r2d2::Error) -> Self {
        DbError(format!("InternalError occured in connection pool: {}", err.to_string()))
    }
}
impl Error for DbError {}

#[derive(Clone)]
pub struct UserDatabase(MysqlPool);
impl UserDatabase {
    pub fn new(url: &String) -> Self {
        Self {
            0: Pool::builder()
                    .max_size(15)
                    .build(ConnectionManager::<MysqlConnection>::new(url))
                    .expect("fail to establish connection pool")
        }
    }
    pub async fn query_by_username(&self, username: &String) -> Result <User, DbError> {
        let conn = self.0.get()?;
        users::table
            .filter(users::username.eq(username))
            .first :: <User> (&conn)
            .map_err(|e| { DbError::from(e) })
    }
    pub async fn query_by_userid(&self, userid: i32) -> Result <User, DbError> {
        let conn = self.0.get()?;
        users::table
            .filter(users::id.eq(userid))
            .first(&conn)
            .map_err(|e| { DbError::from(e) })
    }
    pub async fn insert(&self, username: &String, password_hash: &String, email: &String) -> Result <User, DbError> {
        let conn = self.0.get()?;
        conn.transaction(|| {
            let new_user = NewUser {
                username: username,
                password_hash: password_hash,
                email: email,
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(&conn) ?;
            let inserted_user = users::table
                .order(users::id.desc())
                .first(&conn);
            inserted_user
        }) .map_err(|e| { DbError::from(e) })
    }
}

