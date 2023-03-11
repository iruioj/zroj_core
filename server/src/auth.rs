//! Auth 模块负责用户的鉴权.

use std::{collections::HashMap, sync::RwLock};

use actix_web::{
    error::{self, ErrorBadRequest},
    post, web, HttpResponse,
};
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    /// 用户名
    username: String,
    /// 密码的哈希值（不要明文传递）
    passwd_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// 用于标识本次登陆的会话
    session_id: String,
}

#[post("/login")]
async fn login(
    info: web::Json<LoginPayload>,
    auth_data: web::Data<AuthMap>,
) -> actix_web::Result<web::Json<LoginResponse>> {
    eprintln!("login request: {:?}", info);

    if auth_data.get(&info.username)? != info.passwd_hash {
        return Err(error::ErrorBadRequest("密码不正确"));
    }
    let session_id = "some id string".to_string();
    Ok(web::Json(LoginResponse { session_id }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPayload {
    /// 邮箱
    email: String,
    /// 用户名
    username: String,
    /// 密码的哈希值（不要明文传递）
    passwd_hash: String,
}

#[post("/register")]
async fn register(
    info: web::Json<RegisterPayload>,
    auth_data: web::Data<AuthMap>,
) -> actix_web::Result<HttpResponse> {
    eprintln!("register req: {:?}", &info);

    let RegisterPayload {
        username,
        email,
        passwd_hash,
    } = info.0;
    if !email_address::EmailAddress::is_valid(&email) {
        return Err(ErrorBadRequest("邮箱不合法"));
    }
    if let Ok(_) = auth_data.get(&username) {
        return Err(ErrorBadRequest("用户名已存在"));
    }
    auth_data.set(&username, passwd_hash)?;

    Ok(HttpResponse::Ok().body("注册成功"))
}

/// AuthMap 是一个存储用户鉴权信息的样例 struct
pub struct AuthMap(RwLock<HashMap<String, String>>);

impl AuthMap {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::<String, String>::new()))
    }
    /// 根据用户名获取密码哈希
    pub fn get(&self, username: &str) -> actix_web::Result<String> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res = mp
            .get(username)
            .ok_or(error::ErrorBadRequest("用户不存在"))?;
        Ok(res.to_string())
    }
    pub fn set(&self, username: &str, passwd_hash: String) -> actix_web::Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        mp.insert(username.to_string(), passwd_hash);
        Ok(())
    }
}

pub fn service(auth_map: web::Data<AuthMap>) -> actix_web::Scope {
    web::scope("/auth")
        .app_data(auth_map)
        .service(login)
        .service(register)
}
