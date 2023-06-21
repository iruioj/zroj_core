use crate::auth::{AuthInfo, SessionManager};
use crate::data::user::AManager;
use crate::{SessionID, UserID};
use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::{error, get, post, web, HttpResponse};
use server_derive::scope_service;
use serde::{Deserialize, Serialize};

fn validate_username(username: &String) -> actix_web::Result<()> {
    if username.chars().any(|c| !(c.is_alphanumeric() || c == '_')) {
        return Err(error::ErrorBadRequest(
            "username contains invalid character",
        ));
    }
    if username.len() > 20  || username.len() > 6 {
        return Err(error::ErrorBadRequest("length of username should between [6, 20]"));
    }
    Ok(())
}

/// format of login payload
#[derive(Debug, Deserialize)]
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
    session_container: web::Data<SessionManager>,
    user_data_manager: web::Data<AManager>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    validate_username(&payload.username)?;
    eprintln!("login request: {:?}", payload);
    // eprintln!("session_id: {}", session_id.as_simple());
    let user = match user_data_manager
        .query_by_username(&payload.username)
        .await?
    {
        Some(r) => r,
        None => return Err(error::ErrorBadRequest("user does not exist")),
    };
    if !passwd::verify(&user.password_hash, &payload.password_hash) {
        Err(error::ErrorBadRequest("password not correct"))
    } else {
        let id = SessionID::new_v4(); // generate a random session id
        eprintln!("generate new session id {}", id);
        session_container.set(id, AuthInfo { uid: user.id })?;
        session.insert(crate::auth::SESSION_ID_KEY, id)?;
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("username", user.username).path("/").finish())
            .body("login success"))
    }
}

/// format of register payload
#[derive(Debug, Deserialize)]
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
    user_data_manager: web::Data<AManager>,
) -> actix_web::Result<String> {
    eprintln!("handle register");
    validate_username(&payload.username)?;
    eprintln!("register req: {:?}", &payload);
    if !email_address::EmailAddress::is_valid(&payload.email) {
        return Err(error::ErrorBadRequest("Invalid email address"));
    }
    if user_data_manager
        .query_by_username(&payload.username)
        .await?.is_some()
    {
        return Err(error::ErrorBadRequest("User already exists"));
    }
    let user = user_data_manager
        .new_user(&payload.username, &payload.password_hash, &payload.email)
        .await?;
    dbg!(user);
    Ok("Registration success".to_string())
}

#[derive(Serialize)]
struct AuthInfoRes {
    username: String,
    email: String,
}
/// 查看当前的鉴权信息，用于菜单栏显示
#[get("/info")]
async fn inspect(
    user_db: web::Data<AManager>,
    user_id: Option<web::ReqData<UserID>>,
) -> actix_web::Result<web::Json<AuthInfoRes>> {
    if let Some(id) = user_id {
        let user = user_db.query_by_userid(*id).await?;
        if let Some(user) = user {
            return Ok(web::Json(AuthInfoRes {
                username: user.username,
                email: user.email,
            }));
        }
    }
    Err(error::ErrorBadRequest("user not found"))
}

#[post("/logout")]
async fn logout(
    session_container: web::Data<SessionManager>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let id = session.get::<SessionID>(crate::auth::SESSION_ID_KEY)?;
    if let Some(id) = id {
        session_container.remove(id)?;
        session.remove(crate::auth::SESSION_ID_KEY);
        return Ok(HttpResponse::Ok().body("logout success"));
    }
    Err(error::ErrorBadRequest("invalid session id"))
}

#[scope_service(path = "/auth")]
pub fn service(session_mgr: SessionManager, user_database: web::Data<AManager>) {
    wrap(crate::auth::middleware::SessionAuth::bypass(
        session_mgr.clone(),
    ));
    app_data(web::Data::new(session_mgr));
    app_data(user_database);
    service(login);
    service(logout);
    service(register);
    service(inspect);
}
