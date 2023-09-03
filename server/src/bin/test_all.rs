//! 约定：放在这里测试的服务也需要写在 gen_docs 里面
use actix_web::{web, HttpServer};
use server::{
    app,
    auth::{middleware::SessionAuth, SessionManager},
    data::{gravatar::GravatarDB, problem_statement::StmtDB, submission::SubmDB},
    dev,
    manager::{one_off::OneOffManager, problem_judger::ProblemJudger},
    mkdata,
};
use tracing_subscriber::{
    filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_thread_names(true)
        .with_filter(filter::filter_fn(|meta| {
            // the smaller, the more prior
            meta.level() <= &tracing::Level::INFO &&
            // too annoying to verbose
            !meta
                .module_path()
                .is_some_and(|s| s.contains("actix_session::middleware"))
        }));

    tracing_subscriber::registry().with(fmt_layer).init();

    let dir = tempfile::tempdir().unwrap();
    tracing::info!("dir = {:?}", dir.path());

    let user_db = dev::test_userdb(dir.path()).await;
    tracing::info!("user_db initialized");

    let stmt_db = mkdata!(
        StmtDB,
        server::data::problem_statement::DefaultDB::new("stmt_data")
    );
    stmt_db
        .insert(0, problem::sample::a_plus_b_statment())
        .await
        .expect("insert problem 0");
    tracing::info!("stmt_db initialized (with problem 0)");

    let ojdata_db = dev::test_ojdata_db(dir.path()).await;
    let oneoff = web::Data::new(OneOffManager::new(dir.path().join("oneoff")));
    let gravatar = mkdata!(
        GravatarDB,
        server::data::gravatar::DefaultDB::new(
            dir.path().join("gravatar"),
            "http://sdn.geekzu.org/avatar/".into()
        )
    );
    let judger = web::Data::new(ProblemJudger::new(dir.path().join("problem_judge")));
    let subm_db = mkdata!(
        SubmDB,
        server::data::submission::DefaultDB::new(dir.path().join("subm_db"))
    );

    // once finish judging, update submission database
    {
        let subm_db = subm_db.clone().into_inner();
        let recv = judger.reciver();
        std::thread::spawn(move || {
            let rt = actix_rt::Runtime::new().expect("init actix runtime");
            loop {
                match recv.recv() {
                    Ok((sid, rep)) => {
                        let r = rt.block_on(async {
                            subm_db.update(&sid, rep).await
                        });
                        if let Err(e) = r {
                            tracing::warn!("update subm_db: {:?}", e)
                        }
                    },
                    Err(_) => {
                        tracing::warn!("update subm_db thread closed");
                        return
                    },
                }
            }
        });
    }

    let session_key = actix_web::cookie::Key::generate();
    let revproxy = web::Data::new(dev::frontend_rev_proxy(3456));

    let addr = "localhost:8080";
    tracing::info!("server listen at http://{addr}");

    let session_container = SessionManager::default();
    HttpServer::new(move || {
        dev::dev_server(session_key.clone(), revproxy.clone()).service(
            web::scope("/api")
                .service(app::auth::service(
                    session_container.clone(),
                    user_db.clone(),
                ))
                .service(
                    app::user::service(user_db.clone(), gravatar.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                )
                .service(
                    app::problem::service(
                        stmt_db.clone(),
                        ojdata_db.clone(),
                        subm_db.clone(),
                        judger.clone(),
                    )
                    .wrap(SessionAuth::require_auth(session_container.clone())),
                )
                .service(
                    app::one_off::service(oneoff.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                )
                .service(
                    app::submission::service(subm_db.clone(), judger.clone())
                        .wrap(SessionAuth::require_auth(session_container.clone())),
                ),
        )
    })
    .bind(addr)?
    .run()
    .await
}
