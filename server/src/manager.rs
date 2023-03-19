/// Trying to integrate a polygon like statement generating process

use std::fs::DirBuilder;
use std::sync::RwLock;
use actix_web::{get, post, web, error::ErrorInternalServerError};
use actix_session::{Session};
use serde::{Serialize, Deserialize};
use crate::{
    auth::{fetch_login_state, SessionContainer, LoginState},
    config::ServerConfig,
    schema::{
        ResponseMsg,
        response_msg,
        ResponseJsonData,
        response_json_data_false,
        response_json_data_true,
    }
};
use crate::auth::UserID;
use crate::problem::*;
type ProblemID = u32;
type GroupID = i32;

/// For page /problem/{pid}, api url /api/problem/{pid}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProblemViewData {
    general_config:  GeneralConfig,
    statement: StatementViewData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
enum StatementViewData {
    /// given source code and do client side render
    Markdown(StatementSource),
    /// previous rendered tex into html
    LaTex(String)
}

/// For page /problem/{pid}/data, api url /api/problem/{pid}/data



#[derive(Debug)]
pub struct ProblemManager {
    locks: Vec <RwLock<()> >,
    /// base directory of each problem
    base_dir: String,
    /// the json file that store problem statement
    statement: String,
    /// the directory that stores problem data
    data_dir: String,
    pid_maximum: ProblemID,
}
impl ProblemManager {
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            locks: (0..config.pid_maximum).map(|_| RwLock::new(())).collect(),
            base_dir: config.problem_base_dir.clone(),
            statement: config.problem_statement.clone(),
            data_dir: config.problem_data_dir.clone(),
            pid_maximum: config.pid_maximum.clone(),
        }
    }
    fn fetch_file(&self, path: &String) -> actix_web::Result <String> {
        std::fs::read_to_string(path).
            map_err(|e| ErrorInternalServerError(e.to_string()))
    }
    fn get_base_dir(&self, pid: ProblemID) -> actix_web::Result <String> {
        let mut s = self.base_dir.clone();
        if let None = s.find("{}") {
            return Err(ErrorInternalServerError("Problem base dir is not correct. {} is required".to_string()));
        }
        s = s.replace("{}", &pid.to_string());
        if let Some(_) = s.find("{}") {
            return Err(ErrorInternalServerError("Problem base dir is not correct. Too many {}s".to_string()));
        }
        Ok(s)
    }
    fn read_statement(&self, pid: ProblemID) -> actix_web:: Result <String> {
        let guard = self.locks[pid as usize]
            .read()
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;
        let dir = self.get_base_dir(pid)? + &self.statement;
        let result = self.fetch_file(&dir)?;
        drop(guard);
        Ok(result)
    }
    fn check_access(&self, pid: ProblemID, uid: UserID) -> actix_web::Result <ProblemAccess> {
        todo!()
    }
    fn fetch_view_data(&self, pid: ProblemID) -> actix_web::Result <ProblemViewData> {
        todo!()
    }
}

#[get("/{pid}")]
async fn view_problem(
    pid: web::Path<ProblemID>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Result <web::Json <ResponseJsonData <ProblemViewData> > > {
    if *pid >= manager.pid_maximum {
        return response_json_data_false("Problem does not exists");
    }
    let login_state = fetch_login_state(&session, &session_container)?;
    if let LoginState::UserID(uid) = login_state {
        if(manager.check_access(*pid, uid)? >= ProblemAccess::View) {
            response_json_data_true(manager.fetch_view_data(*pid)?)
        } else {
            response_json_data_false("You do not have access to this problem")
        }
    } else {
        response_json_data_false("Please login first")
    }
}

/*
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
*/

/// 提供 problem 的网络服务
pub fn service(
    session_containter: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Scope {
    web::scope("/api/problem")
        .app_data(session_containter)
        .app_data(manager)
        .service(view_problem)
        // .service(edit)
}
