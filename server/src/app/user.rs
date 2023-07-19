use crate::{
    data::{
        types::*,
        user::{User, UserDB},
    },
    marker::*,
    UserID,
};
use actix_files::NamedFile;
use actix_web::{
    error::{self, ErrorInternalServerError, Result},
    web,
};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use std::{path::Path, io::Write};

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
) -> Result<String> {
    manager.update(*uid, info.into_inner()).await?;
    Ok("ok".to_string())
}

#[derive(Deserialize, SerdeJsonWithType)]
pub struct GravatarInfo {
    pub email: EmailAddress,
    pub no_cache: Option<bool>,
}

#[api(method = post, path = "/gravatar")]
async fn gravatar(info: JsonBody<GravatarInfo>) -> Result<NamedFile> {
    let mut md5 = Md5::new();
    md5.update(info.email.to_string().to_lowercase());
    let hash = format!("{:x?}", md5.finalize().as_slice());
    let path = Path::new("/gravatar").join(&hash).join(".jpg");
    if !path.exists() || info.no_cache == Some(true) {
        let url = String::from("https://www.gravatar.com/avatar/") + &hash;
        let client = awc::Client::default();
        let req = client.get(&url);
        let mut res = req.send().await.map_err(|e| {
            ErrorInternalServerError(format!(
                "Failed to get image from gravatar, {}, url={}",
                e, url
            ))
        })?;
        if !res.status().is_success() {
            return Err(ErrorInternalServerError(format!(
                "Failed to get image from gravatar, status_code:{}, url={}",
                res.status(),
                url
            )));
        }
        let img = res.body().await.map_err(|e| {
            ErrorInternalServerError(format!(
                "Failed to get image from gravatar, {}, url={}",
                e, url
            ))
        })?;
        let mut f = std::fs::OpenOptions::new().create(true).write(true).open(&path)?;
        f.write_all(&img)?;
    }
    NamedFile::open_async(&path).await.map_err(|e| {
        ErrorInternalServerError(format!(
            "Failed to open {}, {}",
            path.to_string_lossy(),
            e.to_string()
        ))
    })
}

#[scope_service(path = "/user")]
pub fn service(user_database: web::Data<UserDB>) {
    app_data(user_database);
    service(profile);
    service(edit_get);
    service(edit_post);
    service(gravatar);
}
