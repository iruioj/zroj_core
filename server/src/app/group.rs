use crate::{
    auth::UserID,
    data::group::{AManager, GroupUsers},
    problem::GroupID,
};
use actix_web::{
    error::{self, Result},
    get, post, web,
};

#[get("/{gid}")]
async fn group_users(
    gid: web::Path<GroupID>,
    manager: web::Data<AManager>,
) -> Result<web::Json<GroupUsers>> {
    Ok(web::Json(
        manager
            .get_group_users(*gid)
            .await?
            .ok_or(error::ErrorBadRequest("No such group"))?,
    ))
}

#[post("/{gid}/add")]
async fn add_users(
    gid: web::Path<GroupID>,
    users: web::ReqData<Vec<UserID>>,
    manager: web::Data<AManager>,
) -> Result<String> {
    let count = manager.group_insert(*gid, &users).await?;
    Ok(format!("Ok, inserted {} users", count))
}

pub fn service(
    group_db: web::Data<AManager>,
) -> actix_web::Scope<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/group").app_data(group_db).service(group_users)
}
