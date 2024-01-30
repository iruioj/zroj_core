/// This script clear the test database and recreate the tables
fn main() {
    let sql_cfg = server::data::mysql::MysqlConfig {
        user: "test".into(),
        password: "test".into(),
        host: "127.0.0.1".into(),
        port: 3306,
        dbname: "test".into(),
    };
    server::data::mysql::setup_database(&sql_cfg, server::data::mysql::SetupDatabaseFlag::ForceNew)
        .expect("setup mysql database");
    let r = server::data::mysql::run_migrations(sql_cfg);
    r.unwrap();
}
