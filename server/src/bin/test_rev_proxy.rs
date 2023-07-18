//! ZROJ 用户信息
use actix_web::http::KeepAlive;
use actix_web::middleware::Logger;

use actix_web::{web, App, HttpServer};
use server::auth::SessionManager;
use server::data::types::{EmailAddress, Username};
use server::data::user::{self, UserDB};
use server::mkdata;
use server::rev_proxy::RevProxy;

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
    let revproxy = web::Data::new(RevProxy::create("http://localhost:3000").path_trans(|s| {
        if s.starts_with("/api") {
            None
        } else {
            // forward to front-end server
            Some(s.to_string())
        }
    }));

    // SSL config, for https testing
    HttpServer::new(move || {
        // 开发的时候不能使用 cors，不然会出问题
        // let cors = Cors::default()
        //     .allowed_origin("http://zroj.tst")
        //     .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b"zroj.tst"))
        //     .allowed_methods(vec!["GET", "POST", "OPTIONS"])
        //     .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::ORIGIN])
        //     .allowed_header(header::CONTENT_TYPE)
        //     .supports_credentials()
        //     .max_age(3600);

        App::new()
            // .wrap(cors)
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
            .service(web::scope("/api").service(server::app::auth::service(
                session_container.clone(),
                user_db.clone(),
            )))
            // reverse proxy
            // .service(
            //     web::scope("/")
            // )
            .app_data(revproxy.clone())
            .default_service(web::route().to(server::rev_proxy::handler::rev_proxy))
        // .default_service(web::route().to(app::default_route))
    })
    // for better development performance
    .keep_alive(KeepAlive::Os)
    .bind("localhost:8080")?
    .run()
    .await
}
