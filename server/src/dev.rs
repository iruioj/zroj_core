//! 主要用于开发

use crate::data::{
    problem_ojdata::{self, OJDataDB},
    problem_statement,
    problem_statement::StmtDB,
    types::*,
    user,
};
use crate::mkdata;
use crate::rev_proxy::RevProxy;
use actix_http::body::MessageBody;
use actix_web::middleware::Logger;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};
use problem::sample::{a_plus_b_data, a_plus_b_statment};

/// 将非 `/api` 开头的请求转发到 localhost:3000
pub fn frontend_rev_proxy() -> RevProxy {
    RevProxy::create("http://localhost:3000").path_trans(|s| {
        if s.starts_with("/api") {
            None
        } else {
            // forward to front-end server
            Some(s.to_string())
        }
    })
}

/// - 默认将请求转发到前端代理
/// - 日志输出到终端
/// - 启用 SessionMiddleware 用于鉴权
pub fn dev_server(
    session_key: actix_web::cookie::Key,
    frontend_proxy: web::Data<RevProxy>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .app_data(frontend_proxy)
        .default_service(web::route().to(crate::rev_proxy::handler::rev_proxy))
        .wrap(Logger::new(r#"%a "%r" %s "%{Referer}i" %T"#))
        .wrap(
            actix_session::SessionMiddleware::builder(
                actix_session::storage::CookieSessionStore::default(),
                session_key,
            )
            .cookie_secure(false)
            // .cookie_same_site(actix_web::cookie::SameSite::None)
            .cookie_path("/".into())
            // .cookie_http_only(false)
            .build(),
        )
}

/// 存储在文件中的用户数据库
///
/// 预先插入用户名 `testtest`，密码 `testtest` 的用户
pub async fn test_userdb(dir: &std::path::Path) -> web::Data<dyn user::Manager + Send + Sync> {
    let r = mkdata!(
        crate::data::user::UserDB,
        user::DefaultDB::new(dir.join("user_data"))
    );
    // 预先插入一个用户方便测试
    r.new_user(
        &Username::new("testtest").unwrap(),
        &passwd::register_hash("testtest"),
        &EmailAddress::new("test@test.com").unwrap(),
    )
    .await
    .unwrap();
    r
}

/// 用于测试的题面数据库
///
/// 预先插入若干个 A + B problem 的题面
pub async fn test_stmtdb(dir: &std::path::Path) -> web::Data<StmtDB> {
    let stmt_db = mkdata!(
        StmtDB,
        problem_statement::DefaultDB::new(dir.join("ojdata"))
    );
    stmt_db
        .insert(0, a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
        .insert(1, a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
        .insert(2, a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
        .insert(3, a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
        .insert(4, a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
}

pub async fn test_ojdata_db(dir: impl AsRef<std::path::Path>) -> web::Data<OJDataDB> {
    let db = mkdata!(
        OJDataDB,
        problem_ojdata::DefaultDB::new(dir.as_ref().join("stmt_data")).unwrap()
    );

    db.insert(0, a_plus_b_data())
        .await
        .expect("fail to insert A + B Problem data");

    db
}
