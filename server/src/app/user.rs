use crate::{
    data::{
        gravatar::GravatarDB,
        types::*,
        user::{User, UserDB},
    },
    marker::*,
    UserID,
};
use actix_files::NamedFile;
use actix_web::{error, web};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

#[derive(Serialize, TsType)]
struct UserDisplayInfo {
    pub id: u32,
    pub username: Username,
    pub email: EmailAddress,
    pub motto: String,
    pub name: String,
    pub register_time: String,
    pub gender: Gender,
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

#[derive(Deserialize, TsType)]
struct ProfileQuery {
    username: Username,
}
#[api(method = get, path = "")]
async fn profile(
    query: QueryParam<ProfileQuery>,
    manager: ServerData<UserDB>,
) -> JsonResult<UserDisplayInfo> {
    let result = manager.query_by_username(&query.username).await?;
    match result {
        Some(info) => Ok(web::Json(UserDisplayInfo::from(info))),
        None => Err(error::ErrorNotFound("user not exist")),
    }
}

#[derive(Serialize, TsType)]
struct UserEditInfo {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub motto: String,
    pub name: String,
    pub register_time: String,
    pub gender: Gender,
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

#[api(method = get, path = "/edit")]
async fn edit_get(
    uid: web::ReqData<UserID>,
    manager: ServerData<UserDB>,
) -> JsonResult<UserEditInfo> {
    let result = manager.query_by_userid(*uid).await?;
    match result {
        Some(info) => Ok(web::Json(UserEditInfo::from(info))),
        None => Err(error::ErrorNotFound("user not exist")),
    }
}

#[derive(Deserialize, TsType)]
pub struct UserUpdateInfo {
    pub password_hash: Option<String>,
    pub email: Option<EmailAddress>,
    pub motto: Option<String>,
    pub name: Option<String>,
    pub gender: Option<Gender>,
}

impl crate::Override<User> for UserUpdateInfo {
    fn over(self, origin: &mut User) {
        if let Some(pw) = self.password_hash {
            origin.password_hash = pw;
        }
        if let Some(e) = self.email {
            origin.email = e;
        }
        if let Some(m) = self.motto {
            origin.motto = m;
        }
        if let Some(n) = self.name {
            origin.name = n;
        }
        if let Some(g) = self.gender {
            origin.gender = g;
        }
    }
}

#[api(method = post, path = "/edit")]
async fn edit_post(
    uid: web::ReqData<UserID>,
    info: JsonBody<UserUpdateInfo>,
    manager: ServerData<UserDB>,
) -> actix_web::Result<String> {
    manager.update(*uid, info.into_inner()).await?;
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
