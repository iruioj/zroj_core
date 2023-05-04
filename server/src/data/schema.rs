use super::group::GroupUsers;
use crate::{GroupID, UserID};
use diesel::{table, AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};

table! {
    users (id) {
        /// id should be auto increment
        id -> Integer,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
        motto -> Varchar,
        name -> Varchar,
        register_time -> Varchar,
        gender -> Integer,
        groups -> Varchar,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Gender {
    Male = 0,
    Female = 1,
    Others = 2,
    Private = 3,
}
impl Gender {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Male,
            1 => Self::Female,
            2 => Self::Others,
            3 => Self::Private,
            _ => panic!("Invalid gender [from i32]"),
        }
    }
}

/// struct for database query
#[derive(Queryable, Debug, Serialize, Deserialize, Clone, AsChangeset)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub motto: String,
    pub name: String,
    pub register_time: String,
    pub gender: i32,
    pub groups: String,
}
/// struct for database insertion
#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: &'a str,
    pub register_time: String,
    pub groups: String,
    pub gender: i32,
}

/*
#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserUpdate {
    pub password_hash: Option<String>,
    pub email: Option<String>,
    pub motto: Option<String>,
    pub name: Option<String>,
    pub gender: Option<i32>,
}
*/

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub owner: i32,
    pub users: GroupUsers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ProblemAccess {
    /// None
    None = 0,
    /// view problem and test data structure, real data if config.open_data
    View = 1,
    /// view any, edit general
    Edit = 2,
    /// view and edit any, or delete a problem
    Admin = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserGroup {
    User(UserID),
    Group(GroupID),
}

/// problem config, stored in config_path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemConfig {
    pub owner: UserID,
    pub access: Vec<(UserGroup, ProblemAccess)>,
    pub open_source_data: bool,
    pub create_date: String,
}
