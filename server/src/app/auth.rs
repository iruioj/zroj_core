use crate::auth::{SessionData, SessionID, SessionManager};
use crate::data::user::AManager;
use actix_web::{error, post, web};
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
    session_container: web::Data<SessionManager>,
    user_data_manager: web::Data<AManager>,
    session_id: web::ReqData<SessionID>,
) -> actix_web::Result<String> {
    validate_username(&payload.username)?;
    eprintln!("login request: {:?}", payload);
    if let Some(result) = user_data_manager
        .query_by_username(&payload.username)
        .await?
    {
        if result.password_hash != payload.password_hash {
            return Err(error::ErrorBadRequest("Password not correct"));
        } else {
            session_container.set(
                *session_id,
                SessionData {
                    login_state: Some(result.id),
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
    session_container: web::Data<SessionManager>,
    user_data_manager: web::Data<AManager>,
    session_id: Option<web::ReqData<SessionID>>,
) -> actix_web::Result<String> {
    validate_username(&payload.username)?;
    let session_id = session_id.ok_or(error::ErrorInternalServerError(
        "auth guard didn't return an authinfo",
    ))?;
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
        *session_id,
        SessionData {
            login_state: Some(result.id),
        },
    )?;
    Ok("Registration success".to_string())
}

pub fn service(
    session_containter: web::Data<SessionManager>,
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
        .wrap(crate::auth::middleware::RequireSession)
        .app_data(session_containter)
        .app_data(user_database)
        .service(login)
        .service(register)
}
