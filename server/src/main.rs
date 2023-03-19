// use actix_web::dev::Server;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{web, App, HttpServer};
use diesel::{
   r2d2 :: {ConnectionManager, Pool, PooledConnection}, 
   mysql :: {MysqlConnection}, 
};
use crate::auth::SessionContainer;
mod admin;
mod app;
mod auth;
mod problem;
mod config;
mod schema;
mod database;
mod manager;
use config::ServerConfig;
#[macro_use]
extern crate diesel;
type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let session_container = web::Data::new(SessionContainer::new());
    let server_config: ServerConfig = config::load();
    let user_database = web::Data::new(database::UserDatabase::new(&server_config.database_url));
    let manager = web::Data::new(manager::ProblemManager::new(&server_config));
    eprintln!("server listening on http://{}:{}", server_config.host, server_config.port);
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    server_config.secret_key.clone(),
                )
            ).configure(app::new(session_container.clone(), user_database.clone(), manager.clone()))
    })
    .bind((server_config.host, server_config.port))?
    .run()
    .await
}


/*
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
*/