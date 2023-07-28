//! 主要用于开发

use actix_http::body::MessageBody;
use actix_web::middleware::Logger;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};

use crate::data::user;
use crate::data::{problem_statement, types::*};
use crate::mkdata;
use crate::rev_proxy::RevProxy;

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
/// 预先插入 A + B problem 的题面，id = 0
pub async fn test_stmtdb(
    dir: &std::path::Path,
) -> web::Data<dyn problem_statement::Manager + Send + Sync> {
    let stmt_db = mkdata!(
        problem_statement::StmtDB,
        problem_statement::DefaultDB::new(dir.join("stmt_data"))
    );
    use problem::render_data::statement::StmtMeta;
    use problem::render_data::Statement;
    stmt_db
        .insert(
            0,
            Statement {
                statement: problem::render_data::statement::Inner::Standard {
                    legend: r#"这是一道简单题。

你需要从标准输入中读入 $a, b$，请你输出 $a + b$。"#
                        .into(),
                    input_format: "输入数据包括两行，每行一个数，分别表示 $a$ 和 $b$。".into(),
                    output_format: "一行一个整数表示答案。".into(),
                    notes: r#"```cpp
#include<iostream>

using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
```"#
                        .into(),
                    samples: vec![(
                        problem::render_data::statement::IOData {
                            fd: problem::render_data::FileDescriptor::Stdin,
                            content: "1\n2".into(),
                        },
                        problem::render_data::statement::IOData {
                            fd: problem::render_data::FileDescriptor::Stdout,
                            content: "3".into(),
                        },
                    )],
                },
                meta: StmtMeta {
                    title: "A + B Problem".into(),
                    time: None,
                    memory: None,
                    kind: Some(problem::render_data::ProblemKind::Traditional(
                        problem::render_data::IOKind::StdIO,
                    )),
                },
            },
        )
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
}
