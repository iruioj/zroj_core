use judger::JudgeResult;
use serde::{Serialize};
use serde_derive::Deserialize;
use diesel::{Queryable,Insertable,table};
use actix_web::web;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;

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

/// format of custom test post payload
#[derive(Debug, MultipartForm)]
pub struct CustomTestPayload {
    #[multipart]
    /// source file, file name: any.{lang}.{suf}
    pub source: TempFile,
    /// input file
    #[multipart]
    pub input: TempFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomTestResult {
    /// return None if the judging or failed
    pub result: Option <JudgeResult>
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




