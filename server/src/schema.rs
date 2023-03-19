use serde::Serialize;
use serde_derive::Deserialize;
use diesel::{Queryable,Insertable,table};
use actix_web::web;

table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
    }
}

/// struct for database query
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct User {
    /// id should be auto increment
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String
}
/// struct for database insertion
#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser <'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: &'a str,
}

/// format of login payload
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    /// 用户名
    pub username: String,
    /// 密码的哈希值（不要明文传递）
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
}

/// format of register payload
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPayload {
    /// 邮箱
    pub email: String,
    /// 用户名
    pub username: String,
    /// 密码的哈希值（不要明文传递）
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
}

/// format of json msg response
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMsg {
    pub ok: bool,
    /// 用于标识本次登陆的会话
    pub msg: String,
}
pub fn response_msg <T: std::fmt::Display> (ok: bool, msg: T) -> actix_web::Result <web::Json <ResponseMsg> > {
    Ok(web::Json(ResponseMsg { ok, msg: msg.to_string() }))
}

/// format of json data response
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseJsonData {
    pub ok: bool,
    /// 用于标识本次登陆的会话
    pub msg: String,
    pub data: String,
}
pub fn response_json_data <T1,T2> (ok: bool, msg: T1, data: T2) 
    -> actix_web::Result <web::Json <ResponseJsonData> > 
where 
    T1: std::fmt::Display,
    T2: std::fmt::Display,
{
    Ok(web::Json(ResponseJsonData { ok, msg: msg.to_string(), data: data.to_string()}))
}