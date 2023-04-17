use std::{sync::{Arc, RwLock}, collections::HashMap, path::PathBuf};
use actix_web::{Result, error, web};
use judger::{JudgeResult, OneOff};
use crate::{auth::UserID, config::core::CoreConfig};

use super::{judge_queue::JudgeQueue, CodeLang};

#[derive(Debug)]
pub struct CustomTestManager {
    /// base directory of each problem
    base_dir: String,
    state: Arc<RwLock<HashMap<UserID, Option<JudgeResult>>>>,
}
impl CustomTestManager {
    pub fn new(config: &CoreConfig) -> Self {
        Self {
            base_dir: config.problem_base_dir.clone(),
            state: Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn check_userid(&self, uid: &UserID) -> Result<()> {
        if *uid < 0 {
            return Err(error::ErrorInternalServerError("User id too large"));
        }
        Ok(())
    }
    pub fn fetch_result(&self, uid: &UserID) -> Result<Option<JudgeResult>> {
        self.check_userid(uid)?;
        let guard = self.state
            .read()
            .map_err(|_| error::ErrorInternalServerError("Fail to get lock"))?;
        Ok((*guard).get(uid).ok_or(error::ErrorBadRequest("No requested custom test"))?.clone())
    }
    pub fn get_user_folder(&self, uid: &UserID) -> Result<PathBuf> {
        let mut path = PathBuf::new();
        path.push(&self.base_dir);
        path = path.join(uid.to_string());
        let b = path.is_dir();
        if !b {
            std::fs::create_dir(&path).map_err(|_| {
                error::ErrorInternalServerError(format!(
                    "Fail to setup user custom test directory: {}",
                    path.to_string_lossy()
                ))
            })?;
        }
        Ok(path)
    }
}

pub fn start_custom_test(
    manager: web::Data<CustomTestManager>,
    queue: web::Data<JudgeQueue>,
    uid: UserID,
    base: PathBuf,
    source: PathBuf,
    input: PathBuf,
    lang: CodeLang,
) -> Result<()> {
    manager.check_userid(&uid)?;
    let state = manager.state.clone();
    queue.add(move || {
        let mut one = OneOff::new(source, Some(input), lang);
        one.set_wd(base);
        let result = one.exec();
        if let Ok(mut guard) = state.write() {
            let result = match result {
                Ok(result) => Some(result),
                Err(_) => None,
            };
            (*guard).insert(uid, result);
            eprintln!("Fail to write judge result, cannot retrive lock");
        }
    });
    Ok(())
}
