//! Auth 模块负责用户的鉴权.
use actix_session::Session;
use actix_web::{
    error::{self, ErrorInternalServerError},
    post, web, Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::RwLock};

use crate::{data::user::{Manager, AManager}};

type SessionID = uuid::Uuid;
pub type UserID = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginState {
    UserID(UserID),
    NotLoggedIn,
}

// session data for request
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SessionData {
    login_state: LoginState,
}

/// session data container
pub struct SessionContainer(RwLock<HashMap<SessionID, SessionData>>);
impl SessionContainer {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::<SessionID, SessionData>::new()))
    }
    /// 根据用户名获取密码哈希
    fn get(&self, id: SessionID) -> Result<Option<SessionData>> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res: Option<SessionData> = mp.get(&id).map(|d| d.clone());
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
fn fetch_sessionid(
    session: &Session,
    session_container: &web::Data<SessionContainer>,
) -> Result<SessionID> {
    if let Some(sessionid) = session.get::<SessionID>("session-id")? {
        if let Some(_) = session_container.get(sessionid)? {
            return Ok(sessionid);
        }
    }
    let sessionid = SessionID::new_v4(); // generate a random session-id
    session.insert("session-id", sessionid)?;
    session_container.set(
        sessionid,
        SessionData {
            login_state: LoginState::NotLoggedIn,
        },
    )?;
    Ok(sessionid)
}
fn fetch_login_state(
    session: &Session,
    session_container: &web::Data<SessionContainer>,
) -> Result<LoginState> {
    let sessionid = fetch_sessionid(session, session_container)?;
    if let Some(session_data) = session_container.get(sessionid)? {
        Ok(session_data.login_state)
    } else {
        Err(ErrorInternalServerError("Session control failed"))
    }
}
pub fn require_login(
    session: &Session,
    session_container: &web::Data<SessionContainer>,
) -> Result<UserID> {
    let state = fetch_login_state(session, session_container)?;
    match state {
        LoginState::UserID(uid) => Ok(uid),
        _ => Err(error::ErrorUnauthorized("Please login first")),
    }
}

fn validate_username(username: &String) -> actix_web::Result<()> {
    if username.chars().any(|c| !(c.is_alphanumeric() || c == '_')) {
        return Err(error::ErrorBadRequest(
            "Username contains invalid character",
        ));
    }
    if username.len() > 20 {
        return Err(error::ErrorBadRequest("Username is too long"));
    }
    if username.len() < 6 {
        return Err(error::ErrorBadRequest("Username is too long"));
    }
    Ok(())
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

#[post("/login")]
async fn login(
    payload: web::Json<LoginPayload>,
    session: Session,
    session_container: web::Data<SessionContainer>,
    user_data_manager: web::Data<AManager>,
) -> actix_web::Result<String> {
    validate_username(&payload.username)?;
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("login request: {:?}", payload);
    if let Some(result) = user_data_manager
        .query_by_username(&payload.username)
        .await?
    {
        if result.password_hash != payload.password_hash {
            return Err(error::ErrorBadRequest("Password not correct"));
        } else {
            session_container.set(
                sessionid,
                SessionData {
                    login_state: LoginState::UserID(result.id),
                },
            )?;
            return Ok("Login success".to_string());
        }
    } else {
        return Err(error::ErrorBadRequest("User does not exist"));
    }
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

#[post("/register")]
async fn register(
    payload: web::Json<RegisterPayload>,
    session: Session,
    session_container: web::Data<SessionContainer>,
    user_data_manager: web::Data<AManager>,
) -> actix_web::Result<String> {
    validate_username(&payload.username)?;
    let sessionid = fetch_sessionid(&session, &session_container)?;
    eprintln!("register req: {:?}", &payload);
    if !email_address::EmailAddress::is_valid(&payload.email) {
        return Err(error::ErrorBadRequest("Invalid email address"));
    }
    if let Some(_) = user_data_manager
        .query_by_username(&payload.username)
        .await?
    {
        return Err(error::ErrorBadRequest("User already exists"));
    }
    let result = user_data_manager
        .insert(&payload.username, &payload.password_hash, &payload.email)
        .await?;
    session_container.set(
        sessionid,
        SessionData {
            login_state: LoginState::UserID(result.id),
        },
    )?;
    Ok("Registration success".to_string())
}

pub fn service(
    session_containter: web::Data<SessionContainer>,
    user_database: web::Data<AManager>,
) -> actix_web::Scope {
    web::scope("/auth")
        .app_data(session_containter)
        .app_data(user_database)
        .service(login)
        .service(register)
}

#[derive(Clone)]
struct AuthInfo {
    sid: Option<uuid::Uuid>,
    data: Option<SessionData>,
}

pub struct AuthGuard {
    data: web::Data<SessionContainer>,
    require_session_id: bool,
    require_session_data: bool,
}

impl AuthGuard {
    pub fn new(
        data: web::Data<SessionContainer>,
        require_session_id: bool,
        require_session_data: bool,
    ) -> Self {
        Self {
            data,
            require_session_data,
            require_session_id,
        }
    }
}

// https://docs.rs/actix-web/4.3.1/actix_web/guard/index.html
impl actix_web::guard::Guard for AuthGuard {
    fn check(&self, ctx: &actix_web::guard::GuardContext<'_>) -> bool {
        let session = actix_session::SessionExt::get_session(ctx);
        eprintln!("get session");
        if let Ok(Some(sid)) = session.get::<SessionID>("session-id") {
            if let Ok(Some(data)) = self.data.get(sid) {
                ctx.req_data_mut().insert(AuthInfo {
                    sid: Some(sid),
                    data: Some(data),
                });
                return true;
            }
            ctx.req_data_mut().insert(AuthInfo {
                sid: Some(sid),
                data: None,
            });
            if self.require_session_data {
                return false;
            }
        }
        ctx.req_data_mut().insert(AuthInfo {
            sid: None,
            data: None,
        });
        !self.require_session_id
    }
}

// serve_* 表示需要监听的测试
#[cfg(test)]
mod tests;
