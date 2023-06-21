use crate::{
    // manager::_problem::{Metadata, ProblemManager},
    ProblemID, UserID,
    data::{problem_config::AManager, schema::ProblemAccess},
};
use actix_web::{error, get, web, Result};
use server_derive::scope_service;

#[get("/{pid}")]
async fn handle_view_problem(
    pid: web::Path<ProblemID>,
    manager: web::Data<ProblemManager>,
    cfg_mgr: web::Data<AManager>,
    uid: web::ReqData<UserID>,
) -> Result<web::Json<Metadata>> {
    if cfg_mgr.get_access(*pid, *uid).await? >= ProblemAccess::View {
        Ok(web::Json(manager.get_metadata(*pid)?))
    } else {
        Err(error::ErrorForbidden("You do not have access to this problem"))
    }
}

/// 提供 problem 相关服务
#[scope_service(path = "/problem")]
pub fn service(problem_manager: web::Data<ProblemManager>, config_manager: web::Data<AManager>) {
    app_data(problem_manager);
    app_data(config_manager);
    service(handle_view_problem);
}
