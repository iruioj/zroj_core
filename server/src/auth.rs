//! Auth 模块负责用户的鉴权.
use std::{collections::HashMap, sync::RwLock};
use actix_web::{
    error::{self},
    post, web
};
use serde::Serialize;
use serde_derive::Deserialize;
use actix_session::Session;
use crate::schema::{LoginPayload, ResponseMsg, RegisterPayload, response_msg};
use crate::database::UserDatabase;
type SessionID = uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginState {
    UserId(i32),
    NotLoggedIn
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SessionData {
    login_state: LoginState
}

/// session data container
pub struct SessionContainer(RwLock<HashMap<SessionID, SessionData>>);
impl SessionContainer {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::<SessionID, SessionData>::new()))
    }
    /// 根据用户名获取密码哈希
    fn get(&self, id: SessionID) -> actix_web::Result<SessionData> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res = mp
            .get(&id)
            .ok_or(error::ErrorBadRequest("Invalid session id. Please check your browser and clear your cookie"))?;
        Ok(res.clone())
    }
    fn set(&self, id: SessionID, data: SessionData) -> actix_web::Result<()> {
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
pub fn fetch_sessionid(session: &Session, session_container: &web::Data <SessionContainer>) -> actix_web::Result <SessionID> {
    if let Some(sessionid) = session.get::<SessionID> ("session-id")? {
        Ok(sessionid)
    } else {
        let sessionid = SessionID::new_v4(); // generate a random session-id
        session.insert("session-id", sessionid)?;
        session_container.set(sessionid, SessionData{login_state: LoginState::NotLoggedIn})?;
        Ok(sessionid)
    }
}
pub fn fetch_login_state(session: &Session, session_container: &web::Data <SessionContainer>) -> actix_web::Result <LoginState> {
    let sessionid = fetch_sessionid(session, session_container)?;
    Ok(session_container.get(sessionid)?.login_state)
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
        return response_msg(false, format!("Invalid username: {}", msg));
    }
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("login request: {:?}", payload);
    if let Some(result) = user_database.query_by_username(&payload.username).await? {
        if result.password_hash != payload.password_hash { 
            return response_msg(false, "Password not correct");
        } else {
            session_container.set(sessionid, SessionData{login_state : LoginState::UserId(result.id)})?;
            return response_msg(true, "Login success");
        }
    } else {
        return response_msg(false, "User does not exist");
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
        return response_msg(false, format!("Invalid username: {}", msg));
    }
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("register req: {:?}", &payload);
    if !email_address::EmailAddress::is_valid(&payload.email) {
        return response_msg(false, "Invalid email address");
    }
    if let Some(_) = user_database.query_by_username(&payload.username).await? {
        return response_msg(false, "User already exists");
    }
    let result = user_database.insert(&payload.username, &payload.password_hash, &payload.email).await?;
    session_container.set(sessionid, SessionData{login_state : LoginState::UserId(result.id)})?;
    response_msg(true, "Registration success")
}

pub fn service(session_containter: web::Data<SessionContainer>, user_database: web::Data <UserDatabase>) -> actix_web::Scope {
    web::scope("/api/auth")
        .app_data(session_containter)
        .app_data(user_database)
        .service(login)
        .service(register)
}
