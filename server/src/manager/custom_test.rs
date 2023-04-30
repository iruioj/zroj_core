use crate::auth::UserID;
use actix_web::{error, web, Result};
use judger::{lang::LangOption, OneOff, TaskResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use super::judge_queue::JudgeQueue;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum CodeLang {
    #[serde(rename = "gnu_cpp20_o2")]
    GnuCpp20O2,
    #[serde(rename = "gnu_cpp17_o2")]
    GnuCpp17O2,
    #[serde(rename = "gnu_cpp14_o2")]
    GnuCpp14O2,
}

impl LangOption for CodeLang {
    fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton {
        match *self {
            Self::GnuCpp14O2 => judger::lang::gnu_cpp14_o2().build_sigton(source, dest),
            Self::GnuCpp17O2 => judger::lang::gnu_cpp17_o2().build_sigton(source, dest),
            Self::GnuCpp20O2 => judger::lang::gnu_cpp20_o2().build_sigton(source, dest),
        }
    }
}

#[derive(Debug)]
pub struct CustomTestManager {
    /// base directory of each problem
    base_dir: PathBuf,
    state: Arc<RwLock<HashMap<UserID, Option<TaskResult>>>>,
}
impl CustomTestManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn fetch_result(&self, uid: &UserID) -> Result<Option<TaskResult>> {
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
    base: PathBuf,
    source: PathBuf,
    input: PathBuf,
    lang: CodeLang,
) -> Result<()> {
    let state = manager.state.clone();
    queue.add(move || {
        if let Ok(mut guard) = state.write() {
            guard.insert(uid, None);
        }
        let mut one = OneOff::new(source, Some(input), lang);
        one.set_wd(base);
        let result = one.exec().unwrap();
        dbg!(&result);
        if let Ok(mut guard) = state.write() {
            guard.insert(uid, Some(result));
        }
    });
    Ok(())
}
