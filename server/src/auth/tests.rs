use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer};

use crate::{auth::AuthInfo, app::default_route};

use super::{AuthGuard, SessionContainer};

/// ReqData 表示 http 请求中的本地数据，比如 AuthGuard 中调用的 req_data_mut
/// 就可以添加数据进来
async fn auth_handle(auth_info: Option<web::ReqData<AuthInfo>>) -> HttpResponse {
    if let Some(auth_info) = auth_info {
        HttpResponse::Accepted().body(format!(
            "with guard, info: {:?} {:?}",
            auth_info.sid, auth_info.data
        ))
    } else {
        HttpResponse::Accepted().body(format!("no guard, no auth_info"))
    }
}

/// 使用 `cargo test serve_auth -- --nocapture` 启动测试
/// 访问一下 `http://127.0.0.1:8080/auth_empty_guard` 看看效果
#[actix_rt::test]
async fn serve_auth() -> Result<(), std::io::Error> {
    let session_container = web::Data::new(SessionContainer::new());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" %T"#,
            ))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::generate(),
            ))
            .app_data(session_container.clone())
            .service(
                web::scope("/auth_empty_guard")
                    // 添加一个 guard
                    .guard(AuthGuard::new(session_container.clone(), false, false))
                    // 然后其实可以加上一堆 service
                    .default_service(web::route().to(auth_handle)),
            )
            // 默认 404
            .default_service(web::route().to(default_route))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
