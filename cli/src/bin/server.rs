//! ZROJ 后端服务器
use std::path::PathBuf;

use actix_web;
use actix_web::web::Data;
use actix_web::{cookie::Key, App, HttpServer};
use server::actix_session::{storage::CookieSessionStore, SessionMiddleware};
use server::auth::SessionManager;
use server::data::{
    group::{self, Manager as GroupManager},
    user::{self, Manager as UserManager},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let session_container = SessionManager::new();
    let user_data_manager =
        Data::from(user::FsManager::new(PathBuf::from("/var/users")).to_amanager());
    let group_manager =
        Data::from(group::FsManager::new(PathBuf::from("/var/groups")).to_amanager());
    let problem_manager = Data::new(server::manager::problem::ProblemManager::new(
        "/var/problems/".to_string(),
        "statement.json".to_string(),
        "data/".to_string(),
    ));
    let custom_test_manager = Data::new(server::manager::custom_test::CustomTestManager::new(
        "/var/custom_test/".into(),
    ));
    let judge_queue = Data::new(server::manager::judge_queue::JudgeQueue::new(8));
    let host = "127.0.0.1".to_string();
    let port = 8080;
    eprintln!("server listening on http://{}:{}", host, port);
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new(
                r#"%a %t "%r" %s "%{Referer}i" %T"#,
            ))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::generate(),
            ))
            .configure(server::app::new(
                session_container.clone(),
                user_data_manager.clone(),
                group_manager.clone(),
                problem_manager.clone(),
                custom_test_manager.clone(),
                judge_queue.clone(),
            ))
    })
    .bind((host, port))?
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
        let sample_auth_data = Data::new(AuthMap::new());
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
