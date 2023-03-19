use std::fs::DirBuilder;
use std::sync::RwLock;
use actix_web::{get, post, web};
use actix_session::{Session};
use crate::{
    auth::{fetch_login_state, SessionContainer},
    config::ServerConfig,
    schema::{
        ResponseMsg,
        response_msg,
        ResponseJsonData,
        response_json_data,
    }
};

enum ProblemAccess {
    None = 0,
    View = 1,
    Edit = 2,
}

#[derive(Debug)]
pub struct ProblemManager {
    lock: Vec <RwLock<()> >,
    /// base directory of each problem
    base_dir: String,
    /// the json file that store problem statement, parsed by fronted
    /// server will only treat this data as a string
    statement: String,
    /// the directory that stores problem data
    data_dir: String,
    pid_maximum: u32,
}
impl ProblemManager {
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            lock: (0..config.pid_maximum).map(|_| RwLock::new(())).collect(),
            base_dir: config.problem_base_dir.clone(),
            statement: config.problem_statement.clone(),
            data_dir: config.problem_data_dir.clone(),
            pid_maximum: config.pid_maximum.clone(),
        }
    }
}

#[get("/{pid}")]
async fn view(
    pid: web::Path<u32>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Result <web::Json <ResponseJsonData> > {
    if *pid >= manager.pid_maximum {
        return response_json_data(false, "Problem does not exists", "");
    }
    let uid = fetch_login_state(&session, &session_container)?;
    todo!()
}

#[get("/{pid}/edit")]
async fn edit(
    pid: web::Path<u32>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Result <web::Json <ResponseJsonData> > {
    if *pid >= manager.pid_maximum {
        return response_json_data(false, "Problem does not exists", "");
    }
    let uid = fetch_login_state(&session, &session_container)?;
    todo!()
}

/// 提供 problem 的网络服务
pub fn service(
    session_containter: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Scope {
    web::scope("/api/problem")
        .app_data(session_containter)
        .app_data(manager)
        .service(view)
        .service(edit)
}
