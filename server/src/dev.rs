//! 主要用于开发

use std::sync::Arc;

use crate::data::{
    file_system::FileSysDb,
    mysql::MysqlDb,
    problem_ojdata::{self, OJDataDB},
    problem_statement,
    types::*,
    user,
};
use crate::mkdata;
use crate::rev_proxy::RevProxy;
use actix_http::body::MessageBody;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};
use problem::sample::{a_plus_b_data, a_plus_b_statment};
use tracing_actix_web::TracingLogger;

/// 将非 `/api` 开头的请求转发到 localhost:3000
pub fn frontend_rev_proxy(port: u16) -> RevProxy {
    RevProxy::create(format!("http://localhost:{port}")).path_trans(|s| {
        if s.starts_with("/api") {
            None
        } else {
            // forward to front-end server
            Some(s.to_string())
        }
    })
}

/// - register a default service that forward unmatched request to frontend server
/// - authenticate using SessionMiddleware
pub fn dev_server(
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
        .app_data(web::Data::new(awc::Client::new()))
        .default_service(web::route().to(crate::rev_proxy::handler::rev_proxy))
        .wrap(TracingLogger::default())
}

/// 存储在文件中的用户数据库
///
/// 预先插入用户名 `testtest`，密码 `testtest` 的用户
pub fn test_userdb(mysqldb: &MysqlDb) -> web::Data<dyn user::Manager + Send + Sync> {
    let db = user::Mysql::new(mysqldb);
    let r = mkdata!(crate::data::user::UserDB, db);
    // 预先插入一个用户方便测试
    if r.query_by_username(&Username::new("testtest").unwrap())
        .is_err()
    {
        let user = r
            .new_user(
                &Username::new("testtest").unwrap(),
                &passwd::register_hash("testtest"),
                &EmailAddress::new("test@test.com").unwrap(),
            )
            .unwrap();
        tracing::info!(?user, "user 'testtset' added");
    } else {
        tracing::info!("user 'testtset' already exists");
    }
    r
}

/// 用于测试的题面数据库
///
/// 预先插入若干个 A + B problem 的题面
pub fn test_stmtdb(
    mysqldb: &MysqlDb,
    filesysdb: &FileSysDb,
) -> web::Data<problem_statement::Mysql> {
    let stmt_db = problem_statement::Mysql::new(mysqldb, filesysdb);
    if stmt_db.get(1).is_err() {
        let id = stmt_db
            .insert_new(a_plus_b_statment())
            .expect("fail to insert A + B Problem");
        assert!(id == 1);
    }
    tracing::info!("test statement db initialized");
    web::Data::new(stmt_db)
}

pub async fn test_ojdata_db(filesysdb: &FileSysDb) -> web::Data<OJDataDB> {
    let db = mkdata!(OJDataDB, problem_ojdata::DefaultDB::new(filesysdb).unwrap());

    db.insert(1, a_plus_b_data())
        .expect("fail to insert A + B Problem data");

    db
}

/// logging configuration for development
pub fn logging_setup(max_level: &'static tracing::Level, log_file: Option<String>) {
    use tracing_subscriber::{
        filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };

    let terminal_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_thread_names(true)
        .with_filter(filter::filter_fn(|meta| {
            // let is_invalid_identity = meta
            //     .module_path()
            //     .is_some_and(|s| s.contains("actix_session::session"));

            meta.level() <= max_level // && !from_actix_session
        }));

    let file_log = log_file
        .and_then(|log_file| std::fs::File::create(log_file).ok())
        .map(|file| {
            let file = std::sync::Mutex::new(Arc::new(file));
            tracing_subscriber::fmt::layer()
                .json()
                .with_thread_names(true)
                .with_writer(move || file.lock().unwrap().clone())
                .with_filter(filter::filter_fn(|meta| {
                    // the smaller, the more prior
                    meta.level() <= max_level &&
            // too annoying to verbose
            !meta
                .module_path()
                .is_some_and(|s| s.contains("actix_session::session"))
                }))
        });
    tracing_subscriber::registry()
        .with(file_log)
        .with(terminal_log)
        .init();
}
