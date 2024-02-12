//! Run this script to prepare database for testing according to `local.server_app_test.json`

use server::data::{
    file_system::{schema::*, FileSysTable},
    mysql::{schema::*, schema_model},
    types::*,
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

    let test_user = schema_model::User {
        id: 1,
        username: Username::new("testtest")?,
        password_hash: passwd::register_hash("testtest"),
        name: "Test".into(),
        email: EmailAddress::new("jy.cat@qq.com")?,
        motto: "Just for test".into(),
        register_time: DateTime::now(),
        gender: Gender::Private,
    };
    eprintln!("insert a user {}", test_user.username);
    mysqldb.upsert(users::table, test_user)?;

    let mut prob_full = problem::sample::a_plus_b_full();
    eprintln!("insert problem {}", prob_full.statement.title);

    mysqldb.upsert(
        problems::table,
        schema_model::Problem {
            id: 1,
            title: prob_full.statement.title,
            meta: JsonStr(prob_full.statement.meta),
        },
    )?;

    mysqldb.upsert(
        problem_statements::table,
        schema_model::ProblemStatement {
            id: 1,
            pid: 1,
            content: JsonStr(prob_full.statement.statement.render_mdast()),
        },
    )?;

    eprintln!("insert a problem with pdf statement");

    mysqldb.upsert(
        problems::table,
        schema_model::Problem {
            id: 2,
            title: "PDF statement test".into(),
            meta: JsonStr(Default::default()),
        },
    )?;

    mysqldb.upsert(
        problem_statements::table,
        schema_model::ProblemStatement {
            id: 2,
            pid: 2,
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

    filesysdb.transaction(|ctx| {
        ojdata::conn(ctx).replace(&1, &mut prob_full.data)?;

        Ok(())
    })?;

    eprintln!("insert path/to/test.pdf to global staticdata");
    filesysdb.transaction(|ctx| {
        eprintln!("ctx = {}", ctx.path().display());
        let mut file = std::fs::File::open("crates/server/tests/test.pdf").unwrap();
        global_staticdata::conn(ctx).replace("path/to/test.pdf", &mut file)?;

        Ok(())
    })?;

    eprintln!("done");
    Ok(())
}
