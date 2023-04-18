use diesel::{table, Insertable, Queryable};
use serde::{Serialize, Deserialize};

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
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: &'a str,
}

/*
/// format of json data response
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseJsonData <T> {
    pub ok: bool,
    /// 用于标识本次登陆的会话
    pub msg: String,
    pub data: Option <T>,
}
pub fn response_json_data_false <T1,T2> (msg: T1)
    -> actix_web::Result <web::Json <ResponseJsonData <T2> > >
where
    T1: std::fmt::Display,
{
    Ok(web::Json(ResponseJsonData { ok: false, msg: msg.to_string(), data: None}))
}
pub fn response_json_data_true <T> (data: T)
    -> actix_web::Result <web::Json <ResponseJsonData <T> > >
{
    Ok(web::Json(ResponseJsonData { ok: false, msg: "ok".to_string(), data: Some(data)}))
}
*/
