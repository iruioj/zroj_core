//! ZROJ 用户信息
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web;
use actix_web::{web, App, HttpServer};
use server::app;
use server::auth::middleware::SessionAuth;
use server::auth::SessionManager;
use server::data::types::{EmailAddress, Username};
use server::data::user::{self, Manager};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::default();
    let user_db = web::Data::from(user::FsManager::new(dir.path().join("user_data")).to_amanager());
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

    // SSL config, for https testing
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://zroj.tst")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b"zroj.tst"))
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::ORIGIN])
            .allowed_header(header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::new(r#"%a "%r" %s "%{Referer}i" %T"#))
            .wrap(
                server::actix_session::SessionMiddleware::builder(
                    server::actix_session::storage::CookieSessionStore::default(),
                    session_key.clone(),
                )
                .cookie_secure(false)
                // .cookie_same_site(actix_web::cookie::SameSite::None)
                // .cookie_domain(Some("zroj.tst".into()))
                .cookie_path("/".into())
                // .cookie_http_only(false)
                .build(),
            )
            .service(server::app::auth::service(
                session_container.clone(),
                user_db.clone(),
            ))
            .service(
                app::user::service(user_db.clone())
                    .wrap(SessionAuth::require_auth(session_container.clone())),
            )
            .default_service(web::route().to(app::default_route))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
