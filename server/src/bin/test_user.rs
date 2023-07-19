//! ZROJ 用户信息
use actix_web::{web, HttpServer};
use server::auth::middleware::SessionAuth;
use server::auth::SessionManager;
use server::data::types::{EmailAddress, Username};
use server::data::user::{self, UserDB};
use server::{app, dev, mkdata};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::default();
    let user_db = mkdata!(UserDB, user::DefaultDB::new(dir.path().join("user_data")));
    // 预先插入一个用户方便测试
    user_db
        .new_user(
            &Username::new("testtest").unwrap(),
            &passwd::register_hash("testtest"),
            &EmailAddress::new("test@test.com").unwrap(),
        )
        .await
        .unwrap();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let session_key = actix_web::cookie::Key::generate();
    let revproxy = web::Data::new(dev::frontend_rev_proxy());

    HttpServer::new(move || {
        dev::dev_server(session_key.clone(), revproxy.clone()).service(
            web::scope("/api")
                .service(server::app::auth::service(
                    session_container.clone(),
                    user_db.clone(),
                ))
                .service(
                    app::user::service(user_db.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                ),
        )
    })
    .bind("localhost:8080")?
    .run()
    .await
}
