//! imp struct for different database queries
use crate::schema::{users, User, NewUser};
use crate::{MysqlPool, MysqlPooledConnection};
use actix_web::{error::ErrorInternalServerError, Result};
use diesel::{
    self, prelude::*,
    r2d2::{ConnectionManager, Pool},
    mysql::{MysqlConnection}, 
};

#[derive(Clone)]
pub struct UserDatabase(MysqlPool);
impl UserDatabase {
    pub fn new(url: &String) -> Self {
        Self {
            0: Pool::builder()
                    .max_size(15)
                    .build(ConnectionManager::<MysqlConnection>::new(url))
                    .expect("fail to establish database connection pool")
        }
    }
    async fn get_conn(&self) -> Result <MysqlPooledConnection> {
        self.0.get().map_err(|e| ErrorInternalServerError(format!("Pool connection error: {}", e.to_string())))
    }
    pub async fn query_by_username(&self, username: &String) -> Result <Option <User> > {
        let conn = self.get_conn().await?;
        let result = users::table
            .filter(users::username.eq(username))
            .first :: <User> (&conn);
        match result {
            Ok(user) => Ok(Some(user)),
            Err(e) => {
                match e  {
                    diesel::result::Error::NotFound => Ok(None),
                    _ => Err(ErrorInternalServerError(format!("Database error: {}", e.to_string())))
                }
            }
        }
    }
    pub async fn query_by_userid(&self, userid: i32) -> Result <Option <User> > {
        let conn = self.get_conn().await?;
        let result = users::table
            .filter(users::id.eq(userid))
            .first(&conn);
        match result {
            Ok(user) => Ok(Some(user)),
            Err(e) => {
                match e {
                    diesel::result::Error::NotFound => Ok(None),
                    _ => Err(ErrorInternalServerError(format!("Database error: {}", e.to_string())))
                }
            }
        }
    }
    pub async fn insert(&self, username: &String, password_hash: &String, email: &String) -> Result <User> {
        let conn = self.get_conn().await?;
        conn.transaction(|| {
            let new_user = NewUser {
                username: username,
                password_hash: password_hash,
                email: email,
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(&conn) ?;
            users::table
                .order(users::id.desc())
                .first :: <User> (&conn)
        }).map_err(|e| ErrorInternalServerError(format!("Database Error: {}", e.to_string())))
    }
}

