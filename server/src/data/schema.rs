use diesel::{table, Insertable, Queryable};
use serde::{Deserialize, Serialize};

table! {
    users (id) {
        /// id should be auto increment
        id -> Integer,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
    }
}

/// struct for database query
#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}
/// struct for database insertion
#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: &'a str,
}

/// struct for database query
#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub owner: i32,
    pub users: Vec<i32>,
}
