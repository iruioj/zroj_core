use server::data::{
    mysql::{MysqlConfig, MysqlDb},
    problem_statement::{self, Manager},
};
use store::Handle;

fn test_db() {
    server::dev::logging_setup(&tracing::Level::DEBUG, None);

    // sql setup
    let sql_cfg = MysqlConfig {
        user: "root".into(),
        password: "root".into(),
        host: "127.0.0.1".into(),
        port: 3306,
        dbname: "test".into(),
    };
    let dir = tempfile::TempDir::new().unwrap();
    dbg!(&sql_cfg);

    MysqlDb::setup_new(&sql_cfg).expect("setup mysql database");
    tracing::debug!("setup database");

    let stmt_db =
        problem_statement::Mysql::new(&sql_cfg, Handle::new(dir.path()).join("stmt_assets"));
    tracing::debug!("create stmt database");

    let r = stmt_db
        .insert_new(problem::sample::a_plus_b_statment())
        .unwrap();
    let stmt = stmt_db.get(r).unwrap();
    tracing::info!(?stmt, "get statement")
}
