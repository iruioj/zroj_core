use server::data::mysql::MysqlConfig;

// make sure to ssh -N -L 33060:localhost:3306 root@zhengruioi.com
fn main() {
    std::fs::create_dir_all("target/zroj/stmt_data_assets").unwrap();
    // let assets_handle = store::Handle::new("target/zroj/stmt_data_assets");
    let sql_cfg = MysqlConfig {
        user: "test".into(),
        password: "test".into(),
        host: "127.0.0.1".into(),
        port: 3306,
        dbname: "test".into(),
    };

    let mut db = server::data::mysql::MysqlDb::new(&sql_cfg);
    eprintln!("try connecting to db");
    let mut conn =
        migrator::establish_connection("root", "Zhengruioi2333", "127.0.0.1", 33060, "zroi");
    eprintln!("connection established");

    migrator::zroj::export_problem_data(&mut db, &mut conn, false);
}
