//! ZROJ 用户信息
use actix_web::http::KeepAlive;
use actix_web::{web, HttpServer};
use server::auth::SessionManager;
use server::dev;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::default();
    let user_db = server::dev::test_userdb(dir.path()).await;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let session_key = actix_web::cookie::Key::generate();

    // 不以 /api 开头的请求都视作前端请求
    let revproxy = web::Data::new(dev::frontend_rev_proxy(3456));

    // SSL config, for https testing
    HttpServer::new(move || {
        dev::dev_server(session_key.clone(), revproxy.clone()).service(web::scope("/api").service(
            server::app::auth::service(session_container.clone(), user_db.clone()),
        ))
    })
    // for better development performance
    .keep_alive(KeepAlive::Os)
    .bind("localhost:8080")?
    .run()
    .await
}
