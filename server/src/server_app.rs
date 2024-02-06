use crate::{
    data::{
        self,
        file_system::FileSysDb,
        mysql::{MysqlConfig, MysqlDb},
    },
    manager::{OneOffManager, ProblemJudger},
    utils,
    web::{
        auth::{injector::AuthInjector, AuthStorage},
        services,
    },
};
use actix_web::web::Data;
use std::{net::ToSocketAddrs, path::PathBuf};

/// Create an online judge server application.
pub struct ServerApp<A: ToSocketAddrs> {
    config: ServerAppConfig<A>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerAppConfig<A: ToSocketAddrs> {
    sql_config: MysqlConfig,
    fs_data_root: PathBuf,
    runner_working_root: PathBuf,
    listen_address: A,
    gravatar_cdn_base: String, // e.g. "https://sdn.geekzu.org/avatar/"
}

pub fn test_server_app_cfg() -> ServerAppConfig<String> {
    ServerAppConfig {
        sql_config: MysqlConfig {
            user: "test".into(),
            password: "test".into(),
            host: "127.0.0.1".into(),
            port: 3306,
            dbname: "test".into(),
        },
        fs_data_root: ".data".into(),
        runner_working_root: ".work".into(),
        listen_address: "127.0.0.1:8080".into(),
        gravatar_cdn_base: "https://sdn.geekzu.org/avatar/".into(),
    }
}

impl<A: ToSocketAddrs> ServerApp<A> {
    pub fn new(cfg: ServerAppConfig<A>) -> Self {
        Self { config: cfg }
    }
    pub async fn start(self) -> anyhow::Result<()> {
        let mysqldb = MysqlDb::new(&self.config.sql_config)?;
        let filesysdb = FileSysDb::new(&self.config.fs_data_root)?;

        let user_db = Data::new(data::user::UserDB::new(&mysqldb));
        let stmt_db = Data::new(data::problem_statement::Mysql::new(&mysqldb, &filesysdb));

        let ojdata_db = Data::new(data::problem_ojdata::OJDataDB::new(&filesysdb)?);
        let oneoff = Data::new(OneOffManager::new(
            self.config.runner_working_root.join("oneoff"),
        )?);
        let gravatar = Data::new(data::gravatar::DefaultDB::new(
            &self.config.gravatar_cdn_base,
        ));
        let judger = Data::new(ProblemJudger::new(
            self.config.runner_working_root.join("problem_judge"),
        )?);
        let subm_db = Data::new(data::submission::Mysql::new(&mysqldb));

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

        let revproxy = Data::new(utils::frontend_rev_proxy(3456));

        let addr = "localhost:8080";
        tracing::info!("server listen at http://{addr}");
        println!("server listen at http://{addr}");

        let auth_storage = AuthStorage::default();
        let tlscfg = std::sync::Arc::new(utils::rustls_config());
        Ok(actix_web::HttpServer::new(move || {
            let gclient = Data::new(crate::data::gravatar::GravatarClient::new(tlscfg.clone()));

            crate::utils::dev_server(revproxy.clone()).service(
                actix_web::web::scope("/api")
                    .service(services::auth::service(
                        auth_storage.clone(),
                        user_db.clone(),
                    ))
                    .service(
                        services::user::service(user_db.clone(), gclient, gravatar.clone())
                            .wrap(AuthInjector::require_auth(auth_storage.clone())),
                    )
                    .service(
                        services::problem::service(
                            stmt_db.clone(),
                            ojdata_db.clone(),
                            subm_db.clone(),
                            judger.clone(),
                        )
                        .wrap(AuthInjector::require_auth(auth_storage.clone())),
                    )
                    .service(
                        services::one_off::service(oneoff.clone())
                            .wrap(AuthInjector::require_auth(auth_storage.clone())),
                    )
                    .service(
                        services::submission::service(subm_db.clone(), judger.clone())
                            .wrap(AuthInjector::require_auth(auth_storage.clone())),
                    )
                    .service(services::api_docs::service()),
            )
        })
        .bind(self.config.listen_address)?
        .run()
        .await?)
    }
}
