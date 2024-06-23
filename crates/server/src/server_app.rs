use crate::{
    data::{
        self,
        file_system::FileSysDb,
        mysql::{MysqlConfig, MysqlDb},
    },
    manager, utils,
    web::auth::{AuthInjector, AuthStorage},
};
use actix_web::web::Data;
use anyhow::Context;
use std::{net::ToSocketAddrs, path::PathBuf};

/// Application runtime data (e.g. database connection), can be used for auto migration.
struct ServerAppRuntime {
    mysqldb: MysqlDb,
    filesysdb: FileSysDb,
    user_db: data::databases::user::UserDB,
    subm_db: data::databases::submission::SubmDB,
    stmt_db: data::databases::problem_statement::StmtDB,
    ojdata_db: data::databases::problem_ojdata::OJDataDB,
    ctst_db: data::databases::contest::CtstDB,
}

/// Create an online judge server application.
pub struct ServerApp<A: ToSocketAddrs> {
    config: ServerAppConfig<A>,
    /// runtime data is accessible before starting the HTTP server.
    runtime: Option<ServerAppRuntime>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerAppConfig<A: ToSocketAddrs> {
    sql_config: MysqlConfig,
    fs_data_root: PathBuf,
    runner_working_root: PathBuf,
    listen_address: A,
    gravatar_cdn_base: String, // e.g. "https://sdn.geekzu.org/avatar/"
    frontend_host: String,
}

impl<A> ServerAppConfig<A>
where
    A: ToSocketAddrs + store::SerdeSerialize,
    for<'a> A: store::SerdeDeserialize<'a>,
{
    /// Try to load config file from path. If not successful,
    /// use the default one and save it to the path.
    ///
    /// ```
    /// let app_cfg = server::ServerAppConfig::load_or_save_default(
    ///     "local.server_app_test.json",
    ///     server::test_server_app_cfg,
    /// ).unwrap();
    /// ```
    pub fn load_or_save_default(
        path: impl AsRef<std::path::Path>,
        default: impl FnOnce() -> ServerAppConfig<A>,
    ) -> anyhow::Result<ServerAppConfig<A>> {
        let app_cfg = std::fs::File::open(&path)
            .context("try to open local test config")
            .and_then(|file| {
                tracing::info!("find local test config");
                serde_json::from_reader(file).context("try to deserialize local test config")
            })
            .unwrap_or_else(|_| default());

        let cfg_file = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .context("save local test config")?;
        serde_json::to_writer_pretty(&cfg_file, &app_cfg).context("serialize local test config")?;
        Ok(app_cfg)
    }
}

/// Configuration for testing server application.
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
        frontend_host: "127.0.0.1:3456".into(),
    }
}

impl<A: ToSocketAddrs> ServerApp<A> {
    /// Create a new server instance. Your application should create at most 1 [`ServerApp`] instance.
    pub fn new(cfg: ServerAppConfig<A>) -> Self {
        Self {
            config: cfg,
            runtime: None,
        }
    }
    /// Get the runtime MySQL database connection after preparing data.
    pub fn runtime_mysqldb(&self) -> Option<MysqlDb> {
        self.runtime.as_ref().map(|o| o.mysqldb.clone())
    }
    /// Get the runtime file system database root after preparing data.
    pub fn runtime_filesysdb(&self) -> Option<FileSysDb> {
        self.runtime.as_ref().map(|o| o.filesysdb.clone())
    }
    /// Force reset mysql database (**This will remove the old data!**)
    pub fn reset_mysql_database(&self) -> anyhow::Result<()> {
        data::mysql::setup_database(
            &self.config.sql_config,
            data::mysql::SetupDatabaseFlag::ForceNew,
        )
        .context("force resetting mysql database")?;
        data::mysql::run_migrations(&self.config.sql_config)
    }
    pub fn reset_filesys_database(&self) -> anyhow::Result<()> {
        FileSysDb::setup_new(&self.config.fs_data_root)?;
        Ok(())
    }
    /// Prepare database backend connections and runtime data managers without starting the http server.
    pub fn prepare_data(&mut self) -> anyhow::Result<()> {
        let mysqldb = MysqlDb::new(&self.config.sql_config)?;
        let filesysdb = FileSysDb::new(&self.config.fs_data_root)?;

        let user_db = data::databases::user::UserDB::new(&mysqldb);
        let stmt_db = data::databases::problem_statement::StmtDB::new(&mysqldb, &filesysdb);
        let ojdata_db = data::databases::problem_ojdata::OJDataDB::new(&filesysdb);
        let subm_db = data::databases::submission::SubmDB::new(&mysqldb);
        let ctst_db = data::databases::contest::CtstDB::new(&mysqldb)?;

        self.runtime = Some(ServerAppRuntime {
            mysqldb,
            filesysdb,
            user_db,
            stmt_db,
            subm_db,
            ojdata_db,
            ctst_db,
        });
        Ok(())
    }
    /// Start the http server.
    pub async fn start(mut self) -> anyhow::Result<()> {
        if self.runtime.is_none() {
            self.prepare_data()?;
        }
        let ServerAppRuntime {
            user_db,
            stmt_db,
            subm_db,
            ojdata_db,
            ctst_db,
            ..
        } = self.runtime.unwrap();

        let user_db = Data::new(user_db);
        let stmt_db = Data::new(stmt_db);
        let subm_db = Data::new(subm_db);
        let ojdata_db = Data::new(ojdata_db);
        let ctst_db = Data::new(ctst_db);

        let oneoff = Data::new(manager::OneOffManager::new(
            self.config.runner_working_root.join("oneoff"),
        )?);
        let judger = Data::new(manager::ProblemJudger::new(
            self.config.runner_working_root.join("problem_judge"),
        )?);
        let permission_manager = Data::new(data::PermissionManager::new());

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

        let revproxy = Data::new(utils::frontend_rev_proxy(format!(
            "http://{}",
            self.config.frontend_host
        )));
        let auth_storage = AuthStorage::default();
        let tlscfg = std::sync::Arc::new(utils::rustls_config());

        #[cfg(feature = "record_request")]
        let reqrecord = {
            let record_dir = store::Handle::new(".record");
            let start_time = chrono::Utc::now();
            record_dir.remove_all()?;
            crate::web::req_record::ReqRecord::new(&record_dir, &start_time)
        };

        let httpserver = actix_web::HttpServer::new(move || {
            use crate::web::services::*;

            let gclient = Data::new(crate::web::gravatar::GravatarClient::new(
                self.config.gravatar_cdn_base.as_ref(),
                tlscfg.clone(),
            ));
            let authinject = AuthInjector::require_auth(auth_storage.clone());

            use actix_web::web;
            let app = actix_web::App::new()
                .app_data(revproxy.clone())
                .app_data(web::Data::new(awc::Client::new()))
                .route("/_nuxt/", web::get().to(crate::web::nuxt::nuxt_websocket))
                .default_service(web::route().to(crate::web::rev_proxy::rev_proxy))
                .wrap(tracing_actix_web::TracingLogger::default());

            #[cfg(feature = "record_request")]
            let app = app.wrap(reqrecord.clone());

            let backend = actix_web::web::scope("/api")
                .app_data(permission_manager.clone())
                .service(
                    auth::service(user_db.clone()).wrap(AuthInjector::bypass(auth_storage.clone())),
                )
                .service(user::service(user_db.clone(), gclient).wrap(authinject.clone()))
                .service(
                    problem::service(
                        stmt_db.clone(),
                        ojdata_db.clone(),
                        subm_db.clone(),
                        judger.clone(),
                    )
                    .wrap(authinject.clone()),
                )
                .service(one_off::service(oneoff.clone()).wrap(authinject.clone()))
                .service(
                    submission::service(subm_db.clone(), judger.clone()).wrap(authinject.clone()),
                )
                .service(contest::service(ctst_db.clone()).wrap(authinject.clone()))
                .service(api_docs::service());

            app.service(backend)
        })
        .bind(self.config.listen_address)?;

        tracing::info!("server listen at {:?}", httpserver.addrs());
        println!("server listen at {:?}", httpserver.addrs());

        Ok(httpserver.run().await?)
    }
}
