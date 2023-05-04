use crate::{
    data::{group::AManager, schema::Group},
    GroupID, UserID,
};
use actix_web::{
    error::{self, Result},
    get, post, web,
};

#[get("/{gid}")]
async fn group_info(
    gid: web::Path<GroupID>,
    manager: web::Data<AManager>,
) -> Result<web::Json<Group>> {
    Ok(web::Json(
        manager
            .get_group_info(*gid)
            .await?
            .ok_or(error::ErrorBadRequest("No such group"))?,
    ))
}

#[post("/{gid}/add")]
async fn add_users(
    gid: web::Path<GroupID>,
    uid: web::ReqData<UserID>,
    users: web::Json<Vec<UserID>>,
    manager: web::Data<AManager>,
) -> Result<String> {
    let count = manager.group_insert(*uid, *gid, &users).await?;
    Ok(format!("Ok, inserted {} users", count))
}

#[post("/{gid}/delete")]
async fn delete_user(
    gid: web::Path<GroupID>,
    uid: web::ReqData<UserID>,
    delete_user: web::Json<UserID>,
    manager: web::Data<AManager>,
) -> Result<String> {
    let count = manager.group_delete(*uid, *gid, *delete_user).await?;
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
    web::scope("/group")
        .app_data(group_db)
        .service(group_info)
        .service(delete_user)
        .service(add_users)
}
