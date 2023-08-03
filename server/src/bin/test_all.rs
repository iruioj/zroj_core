//! 约定：放在这里测试的服务也需要写在 gen_docs 里面
use actix_web::{web, HttpServer};
use server::{
    app,
    auth::{middleware::SessionAuth, SessionManager},
    dev, manager::one_off::OneOffManager,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let session_container = SessionManager::default();
    let user_db = dev::test_userdb(dir.path()).await;
    let stmt_db = dev::test_stmtdb(dir.path()).await;
    let ojdata_db = dev::test_ojdata_db(dir.path()).await;
    let custom_test = web::Data::new(OneOffManager::new("/Users/sshwy/zroj_core/oneoff_tmp"));
    eprintln!("job thread id = {:?}", custom_test.handle.thread().id());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let session_key = actix_web::cookie::Key::generate();
    let revproxy = web::Data::new(dev::frontend_rev_proxy());

    let addr = "localhost:8080";
    eprintln!("server listen at http://{addr}");
    HttpServer::new(move || {
        dev::dev_server(session_key.clone(), revproxy.clone()).service(
            web::scope("/api")
                .service(app::auth::service(
                    session_container.clone(),
                    user_db.clone(),
                ))
                .service(
                    app::user::service(user_db.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                )
                .service(
                    app::problem::service(stmt_db.clone(), ojdata_db.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                )
                .service(
                    app::one_off::service(custom_test.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                ),
        )
    })
    .bind(addr)?
    .run()
    .await
}
