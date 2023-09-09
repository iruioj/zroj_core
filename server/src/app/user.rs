use crate::{
    data::{
        gravatar::GravatarDB,
        types::*,
        user::{UserDB, UserEditInfo, UserUpdateInfo, UserDisplayInfo},
    },
    marker::*,
};
use actix_files::NamedFile;
use actix_web::{error, web::Json};
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
    manager: ServerData<UserDB>,
) -> JsonResult<UserDisplayInfo> {
    let result = manager.query_by_username(&query.username)?;
    Ok(Json(UserDisplayInfo::from(result)))
}

#[api(method = get, path = "/edit")]
async fn edit_get(uid: Identity, manager: ServerData<UserDB>) -> JsonResult<UserEditInfo> {
    let result = manager.query_by_userid(*uid)?;
    Ok(Json(UserEditInfo::from(result)))
}

#[api(method = post, path = "/edit")]
async fn edit_post(
    uid: Identity,
    info: JsonBody<UserUpdateInfo>,
    manager: ServerData<UserDB>,
) -> actix_web::Result<String> {
    manager.update(*uid, info.into_inner())?;
    Ok("ok".to_string())
}

#[derive(Deserialize, TsType)]
pub struct GravatarInfo {
    pub email: EmailAddress,
    pub no_cache: Option<bool>,
}

#[api(method = get, path = "/gravatar")]
async fn gravatar(
    info: QueryParam<GravatarInfo>,
    db: ServerData<GravatarDB>,
) -> actix_web::Result<NamedFile> {
    if info.no_cache.unwrap_or(false) {
        db.fetch(&info.email)
            .await
            .map_err(error::ErrorInternalServerError)
    } else {
        db.get(&info.email)
            .await
            .map_err(error::ErrorInternalServerError)
    }
}

#[scope_service(path = "/user")]
pub fn service(user_database: ServerData<UserDB>, gravatar_db: ServerData<GravatarDB>) {
    app_data(user_database);
    app_data(gravatar_db);
    service(profile);
    service(edit_get);
    service(edit_post);
    service(gravatar);
}
