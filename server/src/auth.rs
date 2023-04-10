//! Auth 模块负责用户的鉴权.
use std::{collections::HashMap, sync::RwLock};
use actix_web::{
    error::{
        self,
        ErrorInternalServerError
    },
    Result,
    post, web
};
use serde::Serialize;
use serde_derive::Deserialize;
use actix_session::Session;
use crate::schema::{LoginPayload, ResponseMsg, RegisterPayload, response_msg};
use crate::database::UserDatabase;
type SessionID = uuid::Uuid;
pub type UserID = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginState {
    UserID(UserID),
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
    fn get(&self, id: SessionID) -> Result <Option <SessionData> > {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res: Option <SessionData> = mp
            .get(&id).map(|d| d.clone());
        Ok(res)
    }
    fn set(&self, id: SessionID, data: SessionData) -> Result<()> {
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
pub fn fetch_sessionid(session: &Session, session_container: &web::Data <SessionContainer>) -> Result <SessionID> {
    if let Some(sessionid) = session.get::<SessionID> ("session-id")? {
        if let Some(_) = session_container.get(sessionid)? {
            return Ok(sessionid);
        }
    }
    let sessionid = SessionID::new_v4(); // generate a random session-id
    session.insert("session-id", sessionid)?;
    session_container.set(sessionid, SessionData{login_state: LoginState::NotLoggedIn})?;
    Ok(sessionid)
}
pub fn fetch_login_state(session: &Session, session_container: &web::Data <SessionContainer>) -> Result <LoginState> {
    let sessionid = fetch_sessionid(session, session_container)?;
    if let Some(session_data) = session_container.get(sessionid)? {
        Ok(session_data.login_state)
    } else {
        Err(ErrorInternalServerError("Session control failed"))
    }
}
pub fn require_login(session: &Session, session_container: &web::Data <SessionContainer>) -> Result <UserID> {
    let state = fetch_login_state(session, session_container)?;
    match state {
        LoginState::UserID(uid)=> Ok(uid),
        _ => Err(error::ErrorUnauthorized("Please login first")),
    }
}

fn validate_username(username: &String) -> actix_web::Result <()> {
    if username.chars().any(|c| { !(c.is_alphanumeric() || c == '_') }) {
        return Err(error::ErrorBadRequest("Username contains invalid character"));
    }
    if username.len() > 20 {
        return Err(error::ErrorBadRequest("Username is too long"));
    }
    if username.len() < 6 {
        return Err(error::ErrorBadRequest("Username is too long"));
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
    validate_username(&payload.username)?;
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("login request: {:?}", payload);
    if let Some(result) = user_database.query_by_username(&payload.username).await? {
        if result.password_hash != payload.password_hash { 
            return response_msg(false, "Password not correct");
        } else {
            session_container.set(sessionid, SessionData{login_state : LoginState::UserID(result.id)})?;
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
    validate_username(&payload.username)?;
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("register req: {:?}", &payload);
    if !email_address::EmailAddress::is_valid(&payload.email) {
        return response_msg(false, "Invalid email address");
    }
    if let Some(_) = user_database.query_by_username(&payload.username).await? {
        return response_msg(false, "User already exists");
    }
    let result = user_database.insert(&payload.username, &payload.password_hash, &payload.email).await?;
    session_container.set(sessionid, SessionData{login_state : LoginState::UserID(result.id)})?;
    response_msg(true, "Registration success")
}

pub fn service(session_containter: web::Data<SessionContainer>, user_database: web::Data <UserDatabase>) -> actix_web::Scope {
    web::scope("/api/auth")
        .app_data(session_containter)
        .app_data(user_database)
        .service(login)
        .service(register)
}
