//! ZROJ 后端服务器鉴权测试

use actix_web;
use actix_web::{cookie::Key, web, App, HttpServer};
use server::actix_session::{storage::CookieSessionStore, SessionMiddleware};
use server::auth::SessionManager;
use server::data::user;
use server::data::user::Manager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::new();
    let user_db = web::Data::from(user::FsManager::new(dir.path().join("user_data")).to_amanager());
    let host = "127.0.0.1".to_string();
    let port = 8080;

    eprintln!("server listening on http://{}:{}", host, port);
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a "%r" %s "%{Referer}i" %T"#,
            ))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                    // for test
                    .cookie_secure(false)
                    .build(),
            )
            .service(server::app::auth::service(
                session_container.clone(),
                user_db.clone(),
            ))
            .service(
                // 测试 require_auth
                web::scope("/require_auth")
                    .wrap(server::auth::middleware::SessionAuth::require_auth(
                        session_container.clone(),
                    ))
                    .app_data(user_db.clone())
                    .service(server::app::auth::inspect),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
