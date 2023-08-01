//! 主要用于开发

use actix_http::body::MessageBody;
use actix_web::middleware::Logger;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};
use judger::StoreFile;
use problem::data::OJData;
use problem::{ProblemFullData, StandardProblem};

use crate::data::problem_ojdata::{self, OJDataDB};
use crate::data::problem_statement::StmtDB;
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

pub fn gen_a_plus_b_statment() -> problem::render_data::Statement {
    problem::render_data::Statement {
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
        meta: problem::render_data::statement::StmtMeta {
            title: "A + B Problem".into(),
            time: None,
            memory: None,
            kind: Some(problem::render_data::ProblemKind::Traditional(
                problem::render_data::IOKind::StdIO,
            )),
        },
    }
}
/// 用于测试的题面数据库
///
/// 预先插入 A + B problem 的题面，id = 0
pub async fn test_stmtdb(dir: &std::path::Path) -> web::Data<StmtDB> {
    let stmt_db = mkdata!(
        StmtDB,
        problem_statement::DefaultDB::new(dir.join("stmt_data"))
    );
    stmt_db
        .insert(0, gen_a_plus_b_statment())
        .await
        .expect("fail to insert A + B Problem");
    stmt_db
}

pub async fn test_ojdata_db(dir: impl AsRef<std::path::Path>) -> web::Data<OJDataDB> {
    let db = mkdata!(
        OJDataDB,
        problem_ojdata::DefaultDB::new(dir.as_ref().join("stmt_data")).unwrap()
    );

    db
}

fn gen_a_plus_b_task(a: i32, b: i32) -> problem::problem::traditional::Task {
    problem::problem::traditional::Task {
        input: StoreFile::from_str(a.to_string(), judger::FileType::Plain),
        output: StoreFile::from_str(b.to_string(), judger::FileType::Plain),
    }
}

pub fn gen_test_fulldata() -> ProblemFullData {
    ProblemFullData {
        data: StandardProblem::Traditional(
            OJData::new(problem::problem::traditional::Meta {
                checker: problem::Checker::AutoCmp {
                    float_relative_eps: 0.0,
                    float_absoulte_eps: 0.0,
                },
                time_limit: problem::Elapse::from(1000u64),
                memory_limit: problem::Memory::from(256u64 << 20),
                output_limit: problem::Memory::from(64u64 << 20),
            })
            .set_data(problem::data::Taskset::Subtasks {
                subtasks: vec![
                    problem::data::Subtask {
                        tasks: vec![
                            gen_a_plus_b_task(1, 2),
                            gen_a_plus_b_task(10, 20),
                            gen_a_plus_b_task(100, 200),
                            gen_a_plus_b_task(1000, 2000),
                            gen_a_plus_b_task(10000, 20000),
                        ],
                        meta: (),
                        score: 0.5,
                    },
                    problem::data::Subtask {
                        tasks: vec![
                            gen_a_plus_b_task(-100, 200),
                            gen_a_plus_b_task(-1000, 2000),
                            gen_a_plus_b_task(-10000, 20000),
                        ],
                        meta: (),
                        score: 0.3,
                    },
                    problem::data::Subtask {
                        tasks: vec![gen_a_plus_b_task(-10000, -20000)],
                        meta: (),
                        score: 0.2,
                    },
                ],
                deps: vec![problem::data::DepRelation::new(2, 1)],
            })
            .set_pre(problem::data::Taskset::Tests {
                tasks: vec![
                    gen_a_plus_b_task(1, 2),
                    gen_a_plus_b_task(10, 20),
                    gen_a_plus_b_task(-100, 200),
                ],
            }),
        ),
        statement: gen_a_plus_b_statment(),
        tutorial: problem::render_data::Tutorial {
            tutorial: problem::render_data::tutorial::Inner::Source("题解已经写在题面中".into()),
            meta: problem::render_data::tutorial::TutrMeta {
                origin: None,
                difficulty: Some("简单".into()),
                tags: vec!["IO".into()],
            },
        },
    }
}
