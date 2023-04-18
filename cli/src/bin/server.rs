//! ZROJ 后端服务器
use std::sync::Arc;

use actix_web;
use server::actix_session::{storage::CookieSessionStore, SessionMiddleware};
use server::data::user::{AManager};
use server::{auth::SessionContainer};
use actix_web::{cookie::Key, web, App, HttpServer};
use server::config::core::CoreConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let session_container = web::Data::new(SessionContainer::new());
    let core_config = CoreConfig::new();
    let user_data_manager = 
        web::Data::from(Arc::new(server::data::user::hashmap::HashMap::new(&core_config.user_data_path)) as Arc <AManager>);
    let problem_manager = web::Data::new(server::manager::problem::ProblemManager::new(&core_config));
    let custom_test_manager =
        web::Data::new(server::manager::custom_test::CustomTestManager::new(&core_config));
    let judge_queue = web::Data::new(server::manager::judge_queue::JudgeQueue::new(
        core_config.judge_count,
    ));
    eprintln!(
        "server listening on http://{}:{}",
        core_config.host, core_config.port
    );
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
                problem_manager.clone(),
                custom_test_manager.clone(),
                judge_queue.clone(),
            ))
    })
    .bind((core_config.host, core_config.port))?
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
