use crate::{
    data::{
        schema::{Gender, User},
        user::AManager,
    },
    GroupID, UserID,
};
use actix_web::{
    error::{self, Result},
    get, post, web,
};
use macros::scope_service;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub struct UserQueryPayload {
    userid: Option<UserID>,
    username: Option<String>,
}

#[get("/")]
async fn profile(
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
async fn edit_get(
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
            origin.gender = g as i32;
        }
    }
}

#[post("/edit")]
async fn edit_post(
    uid: web::ReqData<UserID>,
    info: web::Json<UserUpdateInfo>,
    manager: web::Data<AManager>,
) -> Result<String> {
    manager.update(*uid, info.into_inner()).await?;
    Ok("ok".to_string())
}

#[scope_service(path = "/user")]
pub fn service(user_database: web::Data<AManager>) {
    app_data(user_database);
    service(profile);
    service(edit_get);
    service(edit_post);
}

/*-> actix_web::Scope<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/user")
        .app_data(user_database)
        .service(profile)
        .service(edit_get)
        .service(edit_post)
}*/
