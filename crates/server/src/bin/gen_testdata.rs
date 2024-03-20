//! Run this script to prepare database for testing according to `local.server_app_test.json`

use problem::{Elapse, ProblemFullData};
use server::data::{
    file_system::{schema::*, FileSysTable},
    mysql::{schema::*, schema_model},
    types::*,
    ROOT_USER_ID,
};

fn main() -> anyhow::Result<()> {
    let app_cfg = server::ServerAppConfig::load_or_save_default(
        "local.server_app_test.json",
        server::test_server_app_cfg,
    )?;

    eprintln!("app_cfg = {:#?}", app_cfg);

    let mut app = server::ServerApp::new(app_cfg);
    app.reset_mysql_database()?;
    app.reset_filesys_database()?;
    app.prepare_data()?;

    eprintln!("clear database");
    let mysqldb = app.runtime_mysqldb().unwrap();
    let filesysdb = app.runtime_filesysdb().unwrap();

    let root_user = schema_model::User {
        id: ROOT_USER_ID,
        username: Username::new("root")?,
        password_hash: passwd::register_hash("123456"),
        name: "Root".into(),
        email: EmailAddress::new("root@root.com")?,
        motto: "The root user".into(),
        register_time: DateTime::now(),
        gender: Gender::Private,
    };

    assert!(ROOT_USER_ID != 2);
    let test_user = schema_model::User {
        id: 2,
        username: Username::new("testtest")?,
        password_hash: passwd::register_hash("testtest"),
        name: "Test".into(),
        email: EmailAddress::new("jy.cat@qq.com")?,
        motto: "Just for test".into(),
        register_time: DateTime::now(),
        gender: Gender::Private,
    };
    eprintln!("insert root and user {}", test_user.username);
    mysqldb.upsert(users::table, root_user)?;
    mysqldb.upsert(users::table, test_user)?;

    let insert_prob_full = |pid: u32, mut prob_full: ProblemFullData| -> anyhow::Result<()> {
        eprintln!("insert problem {}", prob_full.statement.title);
        mysqldb.upsert(
            problems::table,
            schema_model::Problem {
                id: pid,
                title: prob_full.statement.title,
                meta: JsonStr(prob_full.statement.meta),
            },
        )?;
        mysqldb.upsert(
            problem_statements::table,
            schema_model::ProblemStatement {
                id: pid,
                pid,
                content: JsonStr(prob_full.statement.statement.render_mdast()),
            },
        )?;
        filesysdb.transaction(|ctx| {
            ojdata::conn(ctx).replace(&pid, &mut prob_full.data)?;

            Ok(())
        })?;
        Ok(())
    };

    insert_prob_full(1, problem::sample::a_plus_b_full())?;
    insert_prob_full(2, problem::sample::quine_full())?;

    eprintln!("insert a problem with pdf statement");

    mysqldb.upsert(
        problems::table,
        schema_model::Problem {
            id: 3,
            title: "PDF statement test".into(),
            meta: JsonStr(Default::default()),
        },
    )?;
    mysqldb.upsert(
        problem_statements::table,
        schema_model::ProblemStatement {
            id: 3,
            pid: 3,
            content: JsonStr(
                problem::render_data::statement::Inner::Legacy(format!(
                    r#"You'll see a PDF frame below.

You can use `[pdf](path/to/pdf)` to display PDF (<= {} bytes) in page.

[pdf](path/to/test.pdf)
"#,
                    server::web::services::problem::PDF_INLINE_SIZE
                ))
                .render_mdast(),
            ),
        },
    )?;
    eprintln!("insert path/to/test.pdf to global staticdata");
    filesysdb.transaction(|ctx| {
        let mut file = std::fs::File::open("crates/server/tests/test.pdf").unwrap();
        global_staticdata::conn(ctx).replace("path/to/test.pdf", &mut file)?;

        Ok(())
    })?;

    eprintln!("insert a contest, elapsing 1h, with 3 problems, already started");
    mysqldb.upsert(
        contests::table,
        schema_model::Contest {
            id: 1,
            title: "Contest 1".into(),
            start_time: DateTime::now(),
            end_time: DateTime::now_with_offset_seconds(24 * 3600),
            duration: CastElapse(Elapse::from_sec(3600)),
        },
    )?;
    mysqldb.upsert(
        contest_problems::table,
        schema_model::ContestProblem { cid: 1, pid: 1 },
    )?;
    mysqldb.upsert(
        contest_problems::table,
        schema_model::ContestProblem { cid: 1, pid: 2 },
    )?;
    mysqldb.upsert(
        contest_problems::table,
        schema_model::ContestProblem { cid: 1, pid: 3 },
    )?;

    eprintln!("insert a contest, elapsing 1h, with 2 problems, not started");
    mysqldb.upsert(
        contests::table,
        schema_model::Contest {
            id: 2,
            title: "Contest 2".into(),
            start_time: DateTime::now_with_offset_seconds(2 * 3600),
            end_time: DateTime::now_with_offset_seconds(24 * 3600),
            duration: CastElapse(Elapse::from_sec(3600)),
        },
    )?;
    mysqldb.upsert(
        contest_problems::table,
        schema_model::ContestProblem { cid: 2, pid: 1 },
    )?;
    mysqldb.upsert(
        contest_problems::table,
        schema_model::ContestProblem { cid: 2, pid: 2 },
    )?;

    eprintln!("done");
    Ok(())
}
