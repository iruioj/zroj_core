use anyhow::Context;
use server::data::mysql::MysqlConfig;

// to export problem db: ssh -N -L 33060:localhost:3306 root@zhengruioi.com
fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("target/zroj/stmt_data_assets")?;
    // let assets_handle = store::Handle::new("target/zroj/stmt_data_assets");
    let sql_cfg = MysqlConfig {
        user: "test".into(),
        password: "test".into(),
        host: "127.0.0.1".into(),
        port: 3306,
        dbname: "test".into(),
    };

    let mut db = server::data::mysql::MysqlDb::new(&sql_cfg);
    let mut conn =
        migrator::establish_connection("root", "Zhengruioi2333", "127.0.0.1", 33060, "zroi")?;

    let mut ojdata_db = server::data::problem_ojdata::DefaultDB::new("target/zroj/ojdata")
        .context("init ojdata db")?;

    let ids = migrator::zroj::export_problem_db(&mut db, &mut conn, false)?;
    println!("total migrated: {}", ids.len());
    // let ids = vec![1];

    migrator::zroj::export_problem_ojdata(&mut ojdata_db, &ids[0..10])?;

    Ok(())
}
