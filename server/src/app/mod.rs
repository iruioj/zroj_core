//! app 模块可以创建 OJ 后端的应用路由配置.
pub mod auth;
// pub mod group;
pub mod one_off;
pub mod problem;
pub mod user;

use crate::{
    auth::{middleware::SessionAuth, SessionManager},
    // data::group::AManager as GroupAManager,
    data::{gravatar::GravatarDB, user::UserDB},
    manager::one_off::OneOffManager,
};
use actix_web::{
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
use judger::StoreFile;

/// 默认 404
pub async fn default_route(req: HttpRequest) -> HttpResponse {
    let mut r = String::new();

    r.push_str("Not found\n\n");
    r.push_str(format!("Uri: {}\n", req.uri()).as_str());
    r.push_str(format!("Method: {}\n", req.method()).as_str());
    r.push_str("Headers:\n");
    for (name, val) in req.headers() {
        r.push_str(format!("- {}:{:?}\n", name, val).as_str());
    }
    HttpResponse::NotFound().body(r)
}

/// 返回一个路由配置闭包函数。
///
/// 如果需要更多的依赖数据请加在 new 的参数中
/// 注意 clone() 的调用应当发生在 HttpServer::new 的闭包中，这里不需要
pub fn new(
    session_mgr: SessionManager,
    user_db: web::Data<UserDB>,
    // group_db: web::Data<GroupAManager>,
    gravatar_db: web::Data<GravatarDB>,
    // problem_config_mgr: web::Data<ProblemConfigAManager>,
    // problem_mgr: web::Data<ProblemManager>,
    custom_test_mgr: web::Data<OneOffManager>,
) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        let session_auth = SessionAuth::require_auth(session_mgr.clone());
        app.service(auth::service(session_mgr, user_db.clone()))
            .service(one_off::service(custom_test_mgr).wrap(session_auth.clone()))
            // .service(problem::service(problem_mgr, problem_config_mgr).wrap(session_auth.clone()))
            .service(user::service(user_db.clone(), gravatar_db).wrap(session_auth.clone()))
            // .service(group::service(group_db.clone()).wrap(session_auth))
            .default_service(web::route().to(default_route));
    }
}

/// 将命名格式为 `name.type.suffix` 的文件解析为 StoreFile
fn parse_named_file(nf: &actix_multipart::form::tempfile::TempFile) -> Option<(String, StoreFile)> {
    let binding = nf.file_name.as_ref()?;
    let lang: Vec<&str> = binding.trim().split('.').collect();
    let name = lang.get(0)?;
    let ty = lang.get(1)?;
    let file_type: judger::FileType = serde_json::from_value(serde_json::json!(ty)).ok()?;
    Some((
        name.to_string(),
        StoreFile {
            file: nf.file.reopen().ok()?,
            file_type,
        },
    ))
}
