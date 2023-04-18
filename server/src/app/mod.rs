//! app 模块可以创建 OJ 后端的应用路由配置.

use crate::{
    auth::{self, SessionContainer},
    manager::{self, custom_test::CustomTestManager, problem::ProblemManager},
    data::user::{Manager, AManager},
};
use actix_web::{
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

/// 默认 404
pub async fn default_route(req: HttpRequest) -> HttpResponse {
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
    session_container: web::Data<SessionContainer>,
    user_data_manager: web::Data<AManager>,
    problem_manager: web::Data<ProblemManager>,
    custom_test_manager: web::Data<CustomTestManager>,
    judge_queue: web::Data<manager::judge_queue::JudgeQueue>,
) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut web::ServiceConfig| {
        app.service(manager::service(
            session_container.clone(),
            problem_manager,
            custom_test_manager,
            judge_queue,
        ))
        // api have access to server_config
        // .service(api::service(server_config.clone()))
        .service(auth::service(session_container.clone(), user_data_manager))
        .default_service(web::route().to(default_route));
    }
}
