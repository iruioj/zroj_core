//! Auth 模块负责用户的鉴权.
use actix_web::{
    error::{self},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::RwLock};

pub type SessionID = uuid::Uuid;
pub type UserID = i32;
pub mod middleware;

// session data for request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthInfo {
    pub uid: UserID,
}

/// session data container
pub struct SessionManager(RwLock<HashMap<SessionID, AuthInfo>>);
impl SessionManager {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::<SessionID, AuthInfo>::new()))
    }
    /// 根据用户名获取密码哈希
    pub fn get(&self, id: SessionID) -> Result<Option<AuthInfo>> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res: Option<AuthInfo> = mp.get(&id).map(|d| d.clone());
        Ok(res)
    }
    pub fn set(&self, id: SessionID, data: AuthInfo) -> Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        mp.insert(id, data);
        Ok(())
    }
    pub fn contains_key(&self, id: SessionID) -> Result<bool> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        Ok(mp.contains_key(&id))
    }
}

/*
pub struct RequireSession{
    data: web::Data<SessionManager>
}
impl RequireSession {
    pub fn new(data: web::Data<SessionManager>) -> Self {
        Self {
            data
        }
    }
}
impl actix_web::guard::Guard for RequireSession {
    /// RequireSession automatically create new session id
    /// only returns false if error encountered
    fn check(&self, ctx: &actix_web::guard::GuardContext<'_>) -> bool {
        let session = actix_session::SessionExt::get_session(ctx);
        let res = session.get::<SessionID>("session-id").unwrap_or(None);
        if let Some(id) = res {
            if let Ok(flag) = self.data.contains_key(id) {
                if flag {
                    ctx.req_data_mut().insert(id);
                    return true;
                }
            } else {
                eprintln!("Error encountered in session guard");
                return false;
            }
        }
        let id = SessionID::new_v4(); // generate a random session-id
        if let Err(_) = self.data.set(id,
            SessionData {
                login_state: None
            },
        ) {
            eprintln!("Error encountered in session guard");
            return false;
        }
        ctx.req_data_mut().insert(id);
        return true;
    }
}


/// this guard must be placed after RequireSession
/// returns false if error or not logged in
pub struct RequireLogin{
    data: web::Data<SessionManager>
}
impl RequireLogin {
    pub fn new(data: web::Data<SessionManager>) -> Self {
        Self {
            data
        }
    }
}
impl actix_web::guard::Guard for RequireLogin {
    fn check(&self, ctx: &actix_web::guard::GuardContext<'_>) -> bool {
        let session = actix_session::SessionExt::get_session(ctx);
        if let Some(id) = ctx.req_data().get :: <SessionID> () {
            if let Ok(res) = self.data.get(id.clone()) {
                if let Some(data) = res {
                    match(data.login_state) {
                        Some(uid) => {
                            ctx.req_data_mut().insert(uid);
                            true
                        },
                        None => false,
                    }
                } else {
                    eprintln!("Login guard: didn't find session data on given session id");
                    return false;
                }
            } else {
                eprintln!("Login guard: error encountered");
                return false;
            }
        } else {
            eprintln!("Login guard: session id was required");
            return false;
        }
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

/// test 
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer};

use crate::app::default_route;

use super::{SessionManager, SessionID};

/// ReqData 表示 http 请求中的本地数据，比如 AuthGuard 中调用的 req_data_mut
/// 就可以添加数据进来
async fn auth_handle(auth_info: Option<web::ReqData<SessionID>>) -> HttpResponse {
    if let Some(auth_info) = auth_info {
        HttpResponse::Accepted().body(format!("with guard, info: {}", *auth_info))
    } else {
        HttpResponse::Accepted().body(format!("no guard, no auth_info"))
    }
}

/// 使用 `cargo test serve_auth -- --nocapture` 启动测试
/// 访问一下 `http://127.0.0.1:8080/auth_empty_guard` 看看效果
#[actix_rt::test]
async fn serve_auth() -> Result<(), std::io::Error> {
    let session_container = web::Data::new(SessionManager::new());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" %T"#,
            ))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::generate(),
            ))
            .app_data(session_container.clone())
            .service(
                web::scope("/auth_empty_guard")
                    // 添加一个 guard
                    .wrap(RequireSession::new(session_container.clone()))
                    // 然后其实可以加上一堆 service
                    .default_service(web::route().to(auth_handle)),
            )
            // 默认 404
            .default_service(web::route().to(default_route))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
*/