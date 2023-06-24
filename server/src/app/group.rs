use crate::{
    data::group::{AManager, Group},
    GroupID, UserID,
};
use actix_web::{
    error::{ErrorBadRequest, ErrorForbidden, Result},
    get, post, web,
};
use server_derive::scope_service;

#[get("/{gid}")]
async fn group_info(
    gid: web::Path<GroupID>,
    manager: web::Data<AManager>,
) -> Result<web::Json<Group>> {
    Ok(web::Json(
        manager
            .get_info(*gid)
            .await?
            .ok_or(ErrorBadRequest("No such group"))?,
    ))
}

#[post("/{gid}/add")]
async fn add_users(
    gid: web::Path<GroupID>,
    uid: web::ReqData<UserID>,
    users: web::Json<Vec<UserID>>,
    manager: web::Data<AManager>,
) -> Result<String> {
    if *uid != 0 {
        return Err(ErrorForbidden("only root can manage groups"));
    }
    let count = manager.insert(*gid, &users).await?;
    Ok(format!("Ok, inserted {} users", count))
}

#[post("/{gid}/delete")]
async fn delete_user(
    gid: web::Path<GroupID>,
    uid: web::ReqData<UserID>,
    delete_user: web::Json<UserID>,
    manager: web::Data<AManager>,
) -> Result<String> {
    if *uid != 0 {
        return Err(ErrorForbidden("only root can manage groups"));
    }
    let count = manager.delete(*gid, *delete_user).await?;
    Ok(format!("Ok, inserted {} users", count))
}

#[scope_service(path = "/group")]
pub fn service(group_db: web::Data<AManager>) {
    app_data(group_db);
    service(group_info);
    service(delete_user);
    service(add_users);
}
