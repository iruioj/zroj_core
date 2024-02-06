use crate::{
    auth::Authentication,
    block_it,
    data::{
        gravatar::{DefaultDB, GravatarClient},
        types::*,
        user::{UserDB, UserDisplayInfo, UserEditInfo, UserUpdateInfo},
    },
    marker::*,
};
use actix_http::StatusCode;
use actix_web::{error, web::Json, HttpResponse};
use serde::Deserialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

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
    block_it!(user_db.update(uid, info.into_inner()))?;
    Ok("ok".to_string())
}

type GravatarDB = DefaultDB;

#[derive(Deserialize, TsType)]
pub struct GravatarInfo {
    pub email: EmailAddress,
    pub no_cache: Option<bool>,
}

#[api(method = get, path = "/gravatar")]
async fn gravatar(
    info: QueryParam<GravatarInfo>,
    gclient: ServerData<GravatarClient>,
    db: ServerData<GravatarDB>,
) -> actix_web::Result<HttpResponse> {
    let img_content = db
        .fetch(gclient.into_inner(), &info.email)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(img_content))
}

#[scope_service(path = "/user")]
pub fn service(
    user_database: ServerData<UserDB>,
    gclient: ServerData<GravatarClient>,
    gravatar_db: ServerData<GravatarDB>,
) {
    app_data(user_database);
    app_data(gravatar_db);
    app_data(gclient);
    service(profile);
    service(edit_get);
    service(edit_post);
    service(gravatar);
}
