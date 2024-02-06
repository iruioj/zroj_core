//! 约定：放在这里测试的服务也需要写在 gen_docs 里面
use actix_web::{web, HttpServer};
use server::{
    app,
    auth::{injector::AuthInjector, AuthStorage},
    data::{
        file_system::FileSysDb,
        gravatar::GravatarClient,
        mysql::{MysqlConfig, MysqlDb},
        submission::SubmDB,
    },
    dev,
    manager::{OneOffManager, ProblemJudger},
    mkdata, rustls_config,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // logging setup
    server::dev::logging_setup(&tracing::Level::INFO, Some("runtime.log".into()));

    // fs store setup
    let dir = tempfile::tempdir().unwrap();
    tracing::info!("dir = {:?}", dir.path());

    // sql setup
    let sql_cfg = MysqlConfig {
        user: "test".into(),
        password: "test".into(),
        host: "127.0.0.1".into(),
        port: 3306,
        dbname: "test".into(),
    };
    // by diesel migration we dont need to setup manually
    // server::data::mysql::MysqlDb::setup_new(&sql_cfg).expect("setup mysql database");
    let mysqldb = MysqlDb::new(&sql_cfg);
    let filesysdb = FileSysDb::new(dir.path());

    let user_db = dev::test_userdb(&mysqldb);
    tracing::info!("user_db initialized");

    let stmt_db = dev::test_stmtdb(&mysqldb, &filesysdb);

    let ojdata_db = dev::test_ojdata_db(&filesysdb).await;
    let oneoff = web::Data::new(OneOffManager::new(dir.path().join("oneoff"))?);
    let gravatar = web::Data::new(server::data::gravatar::DefaultDB::new(
        "https://sdn.geekzu.org/avatar/",
    ));
    let judger = web::Data::new(ProblemJudger::new(dir.path().join("problem_judge"))?);
    let subm_db = mkdata!(SubmDB, server::data::submission::Mysql::new(&mysqldb));

    // once finish judging, update submission database
    {
        let subm_db = subm_db.clone().into_inner();
        let recv = judger.reciver();

        // this thread is implicitly detached, thus no resource leak
        std::thread::Builder::new()
            .name("judgereport".into())
            .spawn(move || loop {
                match recv.recv() {
                    Ok((sid, rep)) => {
                        let r = subm_db.update(&sid, rep);
                        if let Err(e) = r {
                            tracing::info!("update subm_db: {:?}", e)
                        }
                    }
                    Err(_) => {
                        tracing::info!("close judge report thread");
                        return;
                    }
                }
            })?;
    }

    let revproxy = web::Data::new(dev::frontend_rev_proxy(3456));

    let addr = "localhost:8080";
    tracing::info!("server listen at http://{addr}");
    println!("server listen at http://{addr}");

    let auth_storage = AuthStorage::default();
    let tlscfg = std::sync::Arc::new(rustls_config());
    HttpServer::new(move || {
        let gclient = web::Data::new(GravatarClient::new(tlscfg.clone()));

        dev::dev_server(revproxy.clone()).service(
            web::scope("/api")
                .service(app::auth::service(auth_storage.clone(), user_db.clone()))
                .service(
                    app::user::service(user_db.clone(), gclient, gravatar.clone())
                        .wrap(AuthInjector::require_auth(auth_storage.clone())),
                )
                .service(
                    app::problem::service(
                        stmt_db.clone(),
                        ojdata_db.clone(),
                        subm_db.clone(),
                        judger.clone(),
                    )
                    .wrap(AuthInjector::require_auth(auth_storage.clone())),
                )
                .service(
                    app::one_off::service(oneoff.clone())
                        .wrap(AuthInjector::require_auth(auth_storage.clone())),
                )
                .service(
                    app::submission::service(subm_db.clone(), judger.clone())
                        .wrap(AuthInjector::require_auth(auth_storage.clone())),
                )
                .service(app::api_docs::service()),
        )
    })
    .bind(addr)?
    .run()
    .await
}
