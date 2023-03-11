//! app 模块可以创建 OJ 后端的应用路由配置.

use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};

use crate::{
    admin,
    auth::{self, AuthMap},
    problem,
};

async fn default_route() -> HttpResponse {
    HttpResponse::NotFound().body("该路径不存在")
}

/// 返回一个路由配置闭包函数。
///
/// 如果需要更多的依赖数据请加在 new 的参数中
/// 注意 clone() 的调用应当发生在 HttpServer::new 的闭包中，这里不需要
pub fn new(sample_auth_data: web::Data<AuthMap>) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut web::ServiceConfig| {
        app.service(problem::service())
            // api have access to server_config
            // .service(api::service(server_config.clone()))
            .service(admin::service())
            .service(auth::service(sample_auth_data))
            .default_service(web::route().to(default_route));
    }
}
