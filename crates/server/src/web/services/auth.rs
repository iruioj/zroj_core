use crate::data::{
    types::{EmailAddress, Username},
    user::UserDB,
};
use crate::web::auth::{
    injector::AuthInjector, AuthInfo, AuthStorage, Authentication, CLIENT_ID_KEY,
};
use crate::ClientID;
use crate::{block_it, marker::*};
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
///
/// Password should be hashed by [`passwd::login_hash`]
#[api(method = post, path = "/login")]
async fn login(
    payload: JsonBody<LoginPayload>,
    auth_storage: ServerData<AuthStorage>,
    user_db: ServerData<UserDB>,
) -> actix_web::Result<HttpResponse> {
    use actix_web::cookie::Cookie;
    tracing::info!("login request: {:?}", payload);
    let username = payload.username.clone();
    let user = block_it!(user_db.query_by_username(&username))?;

    if !passwd::verify(&user.password_hash, &payload.password_hash) {
        Err(error::ErrorBadRequest("password not correct"))
    } else {
        let id = ClientID::new_v4(); // generate a random session id
        tracing::info!("generate new client id {id} for {}", payload.username);
        auth_storage
            .set(id, AuthInfo { uid: user.id })
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok()
            .cookie(
                Cookie::build(CLIENT_ID_KEY, id.to_string())
                    .path("/")
                    .finish(),
            )
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

/// Register a new user. Password should be hashed by [`passwd::register_hash`]
#[api(method = post, path = "/register")]
async fn register(
    payload: JsonBody<RegisterPayload>,
    user_db: ServerData<UserDB>,
) -> actix_web::Result<String> {
    eprintln!("handle register");
    eprintln!("register req: {:?}", &payload);
    let id = block_it! {
        if user_db.query_by_username(&payload.username).is_ok() {
            Err(anyhow::Error::msg("username conflict").into())
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
async fn inspect(user_db: ServerData<UserDB>, auth: Authentication) -> JsonResult<AuthInfoRes> {
    let id = auth.user_id_or_unauthorized()?;
    let user = block_it!(user_db.query_by_userid(id))?;
    Ok(web::Json(AuthInfoRes {
        username: user.username,
        email: user.email,
    }))
}

#[api(method = post, path = "/logout")]
async fn logout(
    auth_storage: ServerData<AuthStorage>,
    auth: Authentication,
) -> actix_web::Result<String> {
    if let Some(id) = auth.client_id() {
        auth_storage
            .remove(id)
            .map_err(error::ErrorInternalServerError)?;
        return Ok("logout success".into());
    }
    Err(error::ErrorBadRequest("invalid session id"))
}

#[scope_service(path = "/auth")]
pub fn service(auth_storage: AuthStorage, user_database: ServerData<UserDB>) {
    wrap(AuthInjector::bypass(auth_storage.clone()));
    app_data(web::Data::new(auth_storage));
    app_data(user_database);
    service(login);
    service(logout);
    service(register);
    service(inspect);
}
