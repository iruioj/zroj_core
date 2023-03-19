//! Auth 模块负责用户的鉴权.
use std::{collections::HashMap, sync::RwLock};
use actix_web::{
    error::{self},
    post, web
};
use serde::Serialize;
use serde_derive::Deserialize;
use actix_session::Session;
use crate::schema::{User, LoginPayload, ResponseMsg, RegisterPayload};
use crate::MysqlPool;
use crate::database::UserDatabase;
type SessionID = uuid::Uuid;
use diesel::prelude::*;
use diesel::{Queryable,Insertable,table};

#[derive(Debug, Serialize, Deserialize, Clone)]
enum SessionData {
    userid(i32),
    not_logged_in
}

/// session data container
pub struct SessionContainer(RwLock<HashMap<SessionID, SessionData>>);
impl SessionContainer {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::<SessionID, SessionData>::new()))
    }
    /// 根据用户名获取密码哈希
    pub fn get(&self, id: SessionID) -> actix_web::Result<SessionData> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res = mp
            .get(&id)
            .ok_or(error::ErrorBadRequest("Invalid session id. Please check your browser and clear your cookie"))?;
        Ok(res.clone())
    }
    pub fn set(&self, id: SessionID, data: SessionData) -> actix_web::Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        mp.insert(id, data);
        Ok(())
    }
}

// or session.get_session_key() instead 
/// fetch a session-id or create a new one
pub fn fetch_session(session: &Session, session_container: &web::Data <SessionContainer>) -> SessionID {
    if let Some(sessionid) = session.get::<SessionID> ("session-id").unwrap() {
        sessionid
    } else {
        let sessionid = SessionID::new_v4(); // generate a random session-id
        session.insert("session-id", sessionid);
        session_container.set(sessionid, SessionData::not_logged_in);
        sessionid
    }
}

fn valid_username(username: &String) -> Result <(), String> {
    if username.chars().any(|c| { !(c.is_alphanumeric() || c == '_') }) {
        return Err(String::from("Username contains invalid character"));
    }
    if username.len() > 20 {
        return Err(String::from("Username is too long"));
    }
    if username.len() < 6 {
        return Err(String::from("Username is too long"));
    }
    Ok(())
}

#[post("/login")]
async fn login(
    payload: web::Json<LoginPayload>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    user_database: web::Data <UserDatabase>,
) -> actix_web::Result <web::Json <ResponseMsg> > {
    if let Err(msg) = valid_username(&payload.username) {
        return Ok(web::Json(ResponseMsg { ok: false, msg: format!("Invalid username: {}", msg) } ));
    }
    let id = fetch_session(&session, &session_container);
    eprintln!("login request: {:?}", payload);
    if let Some(result) = user_database.query_by_username(&payload.username).await? {
        if result.password_hash != payload.password_hash { 
            Ok(web::Json(ResponseMsg { ok: false, msg: String::from("password not correct") }))
        } else {
            Ok(web::Json(ResponseMsg { ok: true, msg: String::from("ok") }))
        }
    } else {
        Ok(web::Json(ResponseMsg { ok: false, msg: String::from("user does not exist") }))
    }
}

#[post("/register")]
async fn register(
    payload: web::Json<RegisterPayload>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    user_database: web::Data <UserDatabase>,
) -> actix_web::Result <web::Json <ResponseMsg> > {
    if let Err(msg) = valid_username(&payload.username) {
        return Ok(web::Json(ResponseMsg { ok: false, msg: format!("Invalid username: {}", msg) } ));
    }
    let sessionid = fetch_session(&session, &session_container);
    eprintln!("register req: {:?}", &payload);
    if !email_address::EmailAddress::is_valid(&payload.email) {
        return Ok(web::Json(ResponseMsg { ok: false, msg: String::from("Invalid email address") } ));
    }
    if let Ok(result) = user_database.query_by_username(&payload.username).await {
        return Ok(web::Json(ResponseMsg { ok: false, msg: String::from("User already exists") } ));
    }
    let result = user_database.insert(&payload.username, &payload.password_hash, &payload.email).await?;
    session_container.set(sessionid, SessionData::userid(result.id))?;
    Ok(web::Json(ResponseMsg { ok: true, msg: String::from("Registration success") }))
}

pub fn service(session_containter: web::Data<SessionContainer>, user_database: web::Data <UserDatabase>) -> actix_web::Scope {
    web::scope("/auth")
        .app_data(session_containter)
        .app_data(user_database)
        .service(login)
        .service(register)
}
