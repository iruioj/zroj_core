use crate::{
    auth::UserID,
    manager::problem::{ProblemManager, ProblemViewData},
    problem::{ProblemAccess, ProblemID},
};
use actix_web::{error, get, web, Result};

#[get("/{pid}")]
async fn handle_view_problem(
    pid: web::Path<ProblemID>,
    manager: web::Data<ProblemManager>,
    uid: web::ReqData<UserID>,
) -> Result<web::Json<ProblemViewData>> {
    if manager.check_access(*pid, *uid)? >= ProblemAccess::View {
        Ok(web::Json(manager.fetch_view_data(*pid)?))
    } else {
        Err(error::ErrorBadRequest("problem not accessible"))
    }
}

/// 提供 problem 相关服务
/// 
/// scope path: `/problem`
pub fn service(
    problem_manager: web::Data<ProblemManager>,
) -> actix_web::Scope<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/problem")
        .app_data(problem_manager)
        .service(handle_view_problem)
}