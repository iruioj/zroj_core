//! app 模块可以创建 OJ 后端的应用路由配置.
mod auth;
mod custom_test;
mod problem;
use crate::{
    auth::SessionManager,
    data::user::AManager,
    manager::{self, custom_test::CustomTestManager, problem::ProblemManager},
};
use actix_web::{
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
pub use auth::service as auth_service;
pub use custom_test::service as custom_test_service;
pub use problem::service as problem_service;

/// 默认 404
async fn default_route(req: HttpRequest) -> HttpResponse {
    let mut r = String::new();

    r.push_str("Not found\n\n");
    r.push_str(format!("Uri: {}\n", req.uri()).as_str());
    r.push_str(format!("Method: {}\n", req.method()).as_str());
    r.push_str("Headers:\n");
    for (name, val) in req.headers() {
        r.push_str(format!("- {}:{:?}\n", name, val).as_str());
    }
    HttpResponse::NotFound().body(r)
}

/// 返回一个路由配置闭包函数。
///
/// 如果需要更多的依赖数据请加在 new 的参数中
/// 注意 clone() 的调用应当发生在 HttpServer::new 的闭包中，这里不需要
pub fn new(
    session_manager: web::Data<SessionManager>,
    user_data_manager: web::Data<AManager>,
    problem_manager: web::Data<ProblemManager>,
    custom_test_manager: web::Data<CustomTestManager>,
    judge_queue: web::Data<manager::judge_queue::JudgeQueue>,
) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut web::ServiceConfig| {
        app.service(custom_test_service(
            session_manager.clone(),
            custom_test_manager,
            judge_queue,
        ))
        .service(auth_service(session_manager.clone(), user_data_manager))
        .service(problem_service(
            session_manager.clone(),
            problem_manager.clone(),
        ))
        .default_service(web::route().to(default_route));
    }
}
