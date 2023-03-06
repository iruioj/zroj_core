use actix_web::dev::Server;
use actix_web::{get, post, Either, web, App, HttpResponse, HttpServer, Responder, Error};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use async_graphql::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;
mod problem;
mod user;
mod api;
mod admin;

// format strings


static f: &str = "/server_config.yaml";

#[derive(Clone)]
pub struct ServerConfig {
    problem_dir: String,
    problem_stmt: String,
    problem_conf: String,
    problem_data_dir: String,
    user_info: String,
    secret_key: Key,
}
impl ServerConfig {
    pub fn new() -> Self {
        // Defaults
        Self {
            problem_dir: String::from("/problem/{}"),
            problem_stmt: String::from("/problem/{}/statement.md"),
            problem_conf: String::from("/problem/{}/config.yaml"),
            problem_data_dir: String::from("/problem/{}/data"),
            user_info: String::from("/users/{}/info"),
            secret_key: Key::generate(),
        }
    }
}

// ** TODO **
fn load_config() -> ServerConfig {
    ServerConfig::new()
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(include_str!("static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_config = load_config();
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    server_config.secret_key.clone(),
                ))
            .service(web::scope("/problem").configure(problem::config))
            .configure(user::config)
            .service(web::scope("/api").app_data(server_config.clone()).configure(api::config))
            // api have access to server_config
            .service(web::scope("/admin").configure(admin::config))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

