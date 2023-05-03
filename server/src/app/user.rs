use super::AManager;
use crate::{
    auth::UserID,
    data::schema::{Gender, User},
    problem::GroupID,
};
use actix_web::{
    error::{self, Result},
    get, post, web,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UserQueryPayload {
    userid: Option<UserID>,
    username: Option<String>,
}

#[derive(Serialize)]
struct UserDisplayInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub motto: String,
    pub name: String,
    pub register_time: String,
    pub gender: Gender,
    pub groups: Vec<GroupID>,
}
impl From<User> for UserDisplayInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            motto: value.motto,
            name: value.name,
            register_time: value.register_time,
            gender: Gender::from_i32(value.gender),
            groups: serde_json::from_str(&value.groups)
                .expect("Group info is not maintained properly"),
        }
    }
}
#[get("/")]
async fn get_display(
    payload: web::Json<UserQueryPayload>,
    manager: web::Data<AManager>,
) -> Result<web::Json<UserDisplayInfo>> {
    let result;
    if let Some(uid) = payload.userid {
        result = manager.query_by_userid(uid).await?;
    } else if let Some(username) = &payload.username {
        result = manager.query_by_username(username).await?;
    } else {
        return Err(error::ErrorBadRequest("Please provide query info"));
    }
    match result {
        Some(info) => Ok(web::Json(UserDisplayInfo::from(info))),
        None => Err(error::ErrorBadRequest("User does not exist")),
    }
}

#[derive(Serialize)]
struct UserEditInfo {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
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
            username: value.username,
            password_hash: value.password_hash,
            email: value.email,
            motto: value.motto,
            name: value.name,
            register_time: value.register_time,
            gender: Gender::from_i32(value.gender),
        }
    }
}

#[get("/edit")]
async fn get_edit(
    uid: web::ReqData<UserID>,
    manager: web::Data<AManager>,
) -> Result<web::Json<UserEditInfo>> {
    let result = manager.query_by_userid(*uid).await?;
    match result {
        Some(info) => Ok(web::Json(UserEditInfo::from(info))),
        None => Err(error::ErrorBadRequest("User does not exist")),
    }
}

#[derive(Deserialize)]
pub struct UserUpdateInfo {
    pub password_hash: Option<String>,
    pub email: Option<String>,
    pub motto: Option<String>,
    pub name: Option<String>,
    pub gender: Option<Gender>,
}

#[post("/edit")]
async fn edit(
    uid: web::ReqData<UserID>,
    info: web::Json<UserUpdateInfo>,
    manager: web::Data<AManager>,
) -> Result<String> {
    manager.update(*uid, &info).await?;
    Ok("ok".to_string())
}
