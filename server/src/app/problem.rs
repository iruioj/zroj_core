use crate::{
    manager::problem::{ProblemManager, ProblemViewData},
    problem::{ProblemAccess, ProblemID}, 
    auth::{UserID, SessionManager, middleware::RequireAuth},
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
        Err(error::ErrorBadRequest(
            "You do not have access to this problem",
        ))
    }
}

/// 提供 manager 的网络服务
pub fn service(
    session_containter: web::Data<SessionManager>,
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
    web::scope("/api")
        .wrap(RequireAuth)
        .app_data(session_containter)
        .app_data(problem_manager)
        .service(handle_view_problem)
}
