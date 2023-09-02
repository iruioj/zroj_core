//! ZROJ 自定义评测

use actix_web::http::header::{self, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::{self};
use actix_web::{web, App, HttpServer};
use server::app;
use server::auth::middleware::SessionAuth;
use server::auth::SessionManager;
use server::manager::one_off::OneOffManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::default();
    let user_db = server::dev::test_userdb(dir.path()).await;
    let custom_test = web::Data::new(OneOffManager::new(dir.path()));

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let session_key = actix_web::cookie::Key::generate();

    // SSL config, for https testing
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(r#"%a "%r" %s "%{Referer}i" %T"#))
            .wrap_fn(|req, srv| {
                use actix_web::dev::Service;
                let m = req.method().to_owned();
                let fut = srv.call(req);
                async move {
                    let mut res: actix_web::dev::ServiceResponse<_> = fut.await?;
                    res.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_static("http://localhost:3000"),
                    );
                    res.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        HeaderValue::from_static("Origin, X-Requested-With, Content-Type, Accept"),
                    );
                    res.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        HeaderValue::from_static("true"),
                    );
                    if m == actix_web::http::Method::OPTIONS {
                        *res.response_mut().status_mut() = StatusCode::ACCEPTED;
                    }
                    Ok(res)
                }
            })
            .wrap(
                server::actix_session::SessionMiddleware::builder(
                    server::actix_session::storage::CookieSessionStore::default(),
                    session_key.clone(),
                )
                .cookie_secure(false)
                // .cookie_same_site(actix_web::cookie::SameSite::None)
                // .cookie_domain(Some("localhost".into()))
                .cookie_path("/".into())
                // .cookie_http_only(false)
                .build(),
            )
            .service(server::app::auth::service(
                session_container.clone(),
                user_db.clone(),
            ))
            .service(
                app::one_off::service(custom_test.clone())
                    .wrap(SessionAuth::require_auth(session_container.clone())),
            )
    })
    .bind("localhost:8080")?
    .run()
    .await
}
