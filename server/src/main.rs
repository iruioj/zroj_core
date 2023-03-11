// use actix_web::dev::Server;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use crate::auth::AuthMap;
mod admin;
mod app;
mod auth;
mod problem;

// format strings

// static f: &str = "/server_config.yaml";

#[derive(Clone)]
pub struct ServerConfig {
    // problem_dir: String,
    // problem_stmt: String,
    // problem_conf: String,
    // problem_data_dir: String,
    // user_info: String,
    secret_key: Key,
}
impl ServerConfig {
    pub fn new() -> Self {
        // Defaults
        Self {
            // problem_dir: String::from("/problem/{}"),
            // problem_stmt: String::from("/problem/{}/statement.md"),
            // problem_conf: String::from("/problem/{}/config.yaml"),
            // problem_data_dir: String::from("/problem/{}/data"),
            // user_info: String::from("/users/{}/info"),
            secret_key: Key::generate(),
        }
    }
}

// ** TODO **
fn load_config() -> ServerConfig {
    ServerConfig::new()
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sample_auth_data = web::Data::new(AuthMap::new());
    let server_config = load_config();

    eprintln!("server listening on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                server_config.secret_key.clone(),
            ))
            .configure(app::new(sample_auth_data.clone()))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body::{self, MessageBody},
        http, test, web, App,
    };
    use serde::Serialize;
    use serde_json::json;

    use super::*;

    fn post_json(uri: &str, body: impl Serialize) -> test::TestRequest {
        test::TestRequest::post().uri(uri).set_json(body)
    }
    async fn body_str(body: impl MessageBody) -> Option<String> {
        body::to_bytes(body)
            .await
            .ok()
            .and_then(|b| String::from_utf8(b.to_vec()).ok())
    }

    #[actix_web::test]
    async fn auth() {
        let sample_auth_data = web::Data::new(AuthMap::new());
        let app = test::init_service(App::new().configure(app::new(sample_auth_data))).await;
        // 尝试登陆不存在的用户
        let mut req = post_json(
            "/auth/login",
            json! ({
                "username": "sshwy",
                "passwd_hash": "hahah"
            }),
        );
        let mut resp = test::call_service(&app, req.to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        assert_eq!("用户不存在", body_str(resp.into_body()).await.unwrap());
        // 尝试注册用户
        req = post_json(
            "/auth/register",
            json!({
                "username": "sshwy",
                "passwd_hash": "hahaha",
                "email": "test@test.com"
            }),
        );
        resp = test::call_service(&app, req.to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!("注册成功", body_str(resp.into_body()).await.unwrap());
        // 尝试登陆
        req = post_json(
            "/auth/login",
            json! ({
                "username": "sshwy",
                "passwd_hash": "hahaha"
            }),
        );
        resp = test::call_service(&app, req.to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
