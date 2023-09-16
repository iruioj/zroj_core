use crate::auth::{AuthInfo, SessionManager};
use crate::data::{
    types::{EmailAddress, Username},
    user::UserDB,
};
use crate::SessionID;
use crate::{block_it, marker::*};
use actix_session::Session;
use actix_web::{error, web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

/// format of login payload
#[derive(Debug, Deserialize, TsType)]
pub struct LoginPayload {
    /// 用户名
    pub username: Username,
    /// 密码的哈希值（不要明文传递）
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
}

/// 用户登陆，需要提供用户名和密码的哈希值
///
/// 如果登陆成功，http 请求头中会返回 cookie
#[api(method = post, path = "/login")]
async fn login(
    payload: JsonBody<LoginPayload>,
    session_container: ServerData<SessionManager>,
    user_db: ServerData<UserDB>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    use actix_web::cookie::Cookie;
    tracing::info!("login request: {:?}", payload);
    let username = payload.username.clone();
    let user = block_it!(user_db.query_by_username(&username))?;

    if !passwd::verify(&user.password_hash, &payload.password_hash) {
        Err(error::ErrorBadRequest("password not correct"))
    } else {
        let id = SessionID::new_v4(); // generate a random session id
        tracing::info!("generate new session id {}", id);
        session_container.set(id, AuthInfo { uid: user.id })?;
        session.insert(crate::auth::SESSION_ID_KEY, id)?;
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("username", user.username).path("/").finish())
            .body("login success"))
    }
}

/// format of register payload
#[derive(Debug, Deserialize, TsType)]
pub struct RegisterPayload {
    /// 邮箱
    pub email: EmailAddress,
    /// 用户名
    pub username: Username,
    /// 密码的哈希值（不要明文传递）
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
}

#[api(method = post, path = "/register")]
async fn register(
    payload: JsonBody<RegisterPayload>,
    user_db: ServerData<UserDB>,
) -> actix_web::Result<String> {
    eprintln!("handle register");
    eprintln!("register req: {:?}", &payload);
    let id = block_it! {
        if user_db.query_by_username(&payload.username).is_ok() {
            Err(crate::data::error::DataError::Conflict)
        } else {
            user_db.new_user(&payload.username, &payload.password_hash, &payload.email)
        }
    }?;
    Ok(format!("successfully register user with id {id}"))
}

#[derive(Serialize, TsType)]
struct AuthInfoRes {
    username: Username,
    email: EmailAddress,
}
/// 查看当前的鉴权信息，用于菜单栏显示
#[api(method = get, path = "/info")]
async fn inspect(
    user_db: ServerData<UserDB>,
    user_id: Option<Identity>,
) -> JsonResult<AuthInfoRes> {
    if let Some(id) = user_id {
        let user = block_it!(user_db.query_by_userid(*id))?;
        return Ok(web::Json(AuthInfoRes {
            username: user.username,
            email: user.email,
        }));
    }
    Err(error::ErrorBadRequest("invalid identity"))
}

#[api(method = post, path = "/logout")]
async fn logout(
    session_container: ServerData<SessionManager>,
    session: Session,
) -> actix_web::Result<String> {
    let id = session.get::<SessionID>(crate::auth::SESSION_ID_KEY)?;
    if let Some(id) = id {
        session_container.remove(id)?;
        session.remove(crate::auth::SESSION_ID_KEY);
        return Ok("logout success".into());
    }
    Err(error::ErrorBadRequest("invalid session id"))
}

#[scope_service(path = "/auth")]
pub fn service(session_mgr: SessionManager, user_database: ServerData<UserDB>) {
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
