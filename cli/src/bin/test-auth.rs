//! ZROJ 后端服务器鉴权测试
use std::sync::Arc;

use actix_web;
use actix_web::{cookie::Key, web, App, HttpServer};
use server::actix_session::{storage::CookieSessionStore, SessionMiddleware};
use server::auth::SessionManager;
use server::data::user::AManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = web::Data::new(SessionManager::new());
    let user_data_manager = web::Data::from(Arc::new(server::data::user::hashmap::HashMap::new(
        dir.path().join("user_data"),
    )) as Arc<AManager>);
    let host = "127.0.0.1".to_string();
    let port = 8080;

    eprintln!("server listening on http://{}:{}", host, port);
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" %T"#,
            ))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                    .cookie_secure(false) // for test
                    .build(),
            )
            .service(server::app::auth_service(
                session_container.clone(),
                user_data_manager.clone(),
            ))
    })
    .bind((host, port))?
    .run()
    .await
}
