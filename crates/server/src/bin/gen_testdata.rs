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

    let mysqldb = app.runtime_mysqldb().unwrap();
    let filesysdb = app.runtime_filesysdb().unwrap();

    let test_user = schema_model::User {
        id: 1,
        username: Username::new("testtest")?,
        password_hash: passwd::register_hash("testtest"),
        name: "Test".into(),
        email: EmailAddress::new("test@test.com")?,
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

    filesysdb.transaction(|ctx| {
        ojdata::conn(ctx).replace(&1, &mut prob_full.data)?;

        Ok(())
    })?;

    eprintln!("done");
    Ok(())
}