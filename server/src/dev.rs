//! 主要用于开发

use actix_http::body::MessageBody;
use actix_web::middleware::Logger;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};

use crate::rev_proxy::RevProxy;

/// 将非 `/api` 开头的请求转发到 localhost:3000
pub fn frontend_rev_proxy() -> RevProxy {
    RevProxy::create("http://localhost:3000").path_trans(|s| {
        if s.starts_with("/api") {
            None
        } else {
            // forward to front-end server
            Some(s.to_string())
        }
    })
}

/// - 默认将请求转发到前端代理
/// - 日志输出到终端
/// - 启用 SessionMiddleware 用于鉴权
pub fn dev_server(
    session_key: actix_web::cookie::Key,
    frontend_proxy: web::Data<RevProxy>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .app_data(frontend_proxy)
        .default_service(web::route().to(crate::rev_proxy::handler::rev_proxy))
        .wrap(Logger::new(r#"%a "%r" %s "%{Referer}i" %T"#))
        .wrap(
            actix_session::SessionMiddleware::builder(
                actix_session::storage::CookieSessionStore::default(),
                session_key,
            )
            .cookie_secure(false)
            // .cookie_same_site(actix_web::cookie::SameSite::None)
            .cookie_path("/".into())
            // .cookie_http_only(false)
            .build(),
        )
}
