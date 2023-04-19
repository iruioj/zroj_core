//! ZROJ 自定义评测
use actix_web;
use actix_web::{cookie::Key, web, App, HttpServer};
use server::actix_session::{storage::CookieSessionStore, SessionMiddleware};
use server::auth::SessionManager;
use server::data::user;
use server::data::user::Manager;
use server::manager::custom_test::CustomTestManager;
use server::manager::judge_queue::JudgeQueue;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::new();
    let user_db = web::Data::from(user::FsManager::new(dir.path().join("user_data")).to_amanager());
    let host = "127.0.0.1".to_string();
    let port = 8080;

    let custom_test = web::Data::new(CustomTestManager::new(dir.path().to_path_buf()));
    let que = web::Data::new(JudgeQueue::new(8));

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
            .service(server::app::custom_test::service(
                custom_test.clone(),
                que.clone(),
            ))
    })
    .bind((host, port))?
    .run()
    .await
}
