use crate::UserID;
use actix_web::{error, web, Result};
use judger::{OneOff, StoreFile, TaskReport};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use super::judge_queue::JudgeQueue;

#[derive(Debug)]
pub struct CustomTestManager {
    /// base directory of each problem
    base_dir: PathBuf,
    state: Arc<RwLock<HashMap<UserID, Option<TaskReport>>>>,
}
impl CustomTestManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn fetch_result(&self, uid: &UserID) -> Result<Option<TaskReport>> {
        let guard = self
            .state
            .read()
            .map_err(|_| error::ErrorInternalServerError("Poisoned lock"))?;
        Ok((*guard)
            .get(uid)
            .ok_or(error::ErrorBadRequest("No requested custom test"))?
            .clone())
    }
    pub fn get_user_folder(&self, uid: &UserID) -> Result<PathBuf> {
        let path = self.base_dir.join(uid.to_string());
        if !path.is_dir() {
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
    source: StoreFile,
    input: StoreFile,
) -> Result<()> {
    let base = manager.get_user_folder(&uid)?;
    let state = manager.state.clone();
    queue.add(move || {
        if let Ok(mut guard) = state.write() {
            guard.insert(uid, None);
        }
        let mut one = OneOff::new(source, Some(input));
        one.set_wd(judger::Handle::new(base));
        let result = one.exec().unwrap();
        dbg!(&result);
        if let Ok(mut guard) = state.write() {
            guard.insert(uid, Some(result));
        }
    });
    Ok(())
}
