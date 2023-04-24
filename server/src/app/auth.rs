use crate::auth::{AuthInfo, SessionID, SessionManager, UserID};
use crate::data::user::AManager;
use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::{error, get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

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
        return Err(error::ErrorBadRequest("Username is too short"));
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
        None => return Err(error::ErrorBadRequest("User does not exist")),
    };
    if !passwd::verify(&user.password_hash, &payload.password_hash) {
        return Err(error::ErrorBadRequest("Password not correct"));
    } else {
        let id = SessionID::new_v4(); // generate a random session-id
        eprintln!("generate new session id {}", id);
        session_container.set(id, AuthInfo { uid: user.id })?;
        session.insert("session-id", id)?;
        return Ok(HttpResponse::Ok()
            .cookie(Cookie::build("username", user.username).path("/").finish())
            .body("login success"));
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
    user_data_manager: web::Data<AManager>,
) -> actix_web::Result<String> {
    eprintln!("handle register");
    validate_username(&payload.username)?;
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
    let user = user_data_manager
        .insert(&payload.username, &payload.password_hash, &payload.email)
        .await?;
    dbg!(user);
    Ok("Registration success".to_string())
}

#[derive(Serialize)]
struct InspectRes {
    session_id: Option<SessionID>,
    user_id: Option<UserID>,
    user: Option<crate::data::schema::User>,
}
/// 查看当前的鉴权信息（主要用于测试）
#[get("/inspect")]
async fn inspect(
    user_db: web::Data<AManager>,
    session_id: Option<web::ReqData<SessionID>>,
    user_id: Option<web::ReqData<UserID>>,
) -> actix_web::Result<web::Json<InspectRes>> {
    let mut res = InspectRes {
        session_id: None,
        user_id: None,
        user: None,
    };
    if let Some(id) = user_id {
        res.user_id = Some(*id);
        res.user = user_db.query_by_userid(*id).await?;
    }
    if let Some(sid) = session_id {
        res.session_id = Some(*sid);
    }
    Ok(web::Json(res))
}

/// scope path: `/auth`
pub fn service(
    session_mgr: SessionManager,
    user_database: web::Data<AManager>,
) -> actix_web::Scope<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/auth")
        .wrap(crate::auth::middleware::SessionAuth::bypass(
            session_mgr.clone(),
        ))
        .app_data(web::Data::new(session_mgr))
        .app_data(user_database)
        .service(login)
        .service(register)
        .service(inspect)
}
