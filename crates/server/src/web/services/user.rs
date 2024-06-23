use crate::{
    block_it,
    data::{mysql::schema_model::User, types::*, databases::user::UserDB},
    marker::*,
    web::auth::Authentication,
    web::gravatar::GravatarClient,
};
use actix_http::StatusCode;
use actix_web::{error, web::Json, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

#[derive(Serialize, TsType)]
pub struct UserDisplayInfo {
    id: u32,
    username: Username,
    email: EmailAddress,
    motto: String,
    name: String,
    register_time: String,
    gender: Gender,
}
impl From<User> for UserDisplayInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            motto: value.motto,
            name: value.name,
            register_time: value.register_time.to_string(),
            gender: value.gender,
        }
    }
}

#[derive(Serialize, TsType)]
pub struct UserEditInfo {
    id: u32,
    username: String,
    email: String,
    motto: String,
    name: String,
    register_time: String,
    gender: Gender,
}

impl From<User> for UserEditInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username.to_string(),
            email: value.email.to_string(),
            motto: value.motto,
            name: value.name,
            register_time: value.register_time.to_string(),
            gender: value.gender,
        }
    }
}

#[derive(Deserialize, TsType)]
pub struct UserUpdateInfo {
    password_hash: Option<String>,
    email: Option<EmailAddress>,
    motto: Option<String>,
    name: Option<String>,
    gender: Option<Gender>,
}

#[derive(Deserialize, TsType)]
struct ProfileQuery {
    username: Username,
}
#[api(method = get, path = "")]
async fn profile(
    query: QueryParam<ProfileQuery>,
    user_db: ServerData<UserDB>,
) -> JsonResult<UserDisplayInfo> {
    let result = block_it!(user_db.query_by_username(&query.username))?;
    Ok(Json(UserDisplayInfo::from(result)))
}

#[api(method = get, path = "/edit")]
async fn edit_get(auth: Authentication, user_db: ServerData<UserDB>) -> JsonResult<UserEditInfo> {
    let uid = auth.user_id_or_unauthorized()?;
    let result = block_it!(user_db.query_by_userid(uid))?;
    Ok(Json(UserEditInfo::from(result)))
}

#[api(method = post, path = "/edit")]
async fn edit_post(
    auth: Authentication,
    info: JsonBody<UserUpdateInfo>,
    user_db: ServerData<UserDB>,
) -> actix_web::Result<String> {
    let uid = auth.user_id_or_unauthorized()?;
    let info = info.into_inner();
    block_it!(user_db.update(
        uid,
        info.password_hash,
        info.email,
        info.motto,
        info.name,
        info.gender
    ))?;
    Ok("ok".to_string())
}

#[derive(Deserialize, TsType)]
pub struct GravatarInfo {
    pub email: EmailAddress,
    pub no_cache: Option<bool>,
}

/// Get the Gravatar image
#[api(method = get, path = "/gravatar")]
async fn gravatar(
    info: QueryParam<GravatarInfo>,
    gclient: ServerData<GravatarClient>,
) -> actix_web::Result<HttpResponse> {
    let img_content = gclient
        .fetch(&info.email)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::build(StatusCode::OK)
        // https://imagekit.io/blog/ultimate-guide-to-http-caching-for-static-assets/
        .insert_header(("Cache-Control", "public, max-age=15552000"))
        .content_type("image/jpeg")
        .body(img_content))
}

#[scope_service(path = "/user")]
pub fn service(user_database: ServerData<UserDB>, gclient: ServerData<GravatarClient>) {
    app_data(user_database);
    app_data(gclient);
    service(profile);
    service(edit_get);
    service(edit_post);
    service(gravatar);
}
