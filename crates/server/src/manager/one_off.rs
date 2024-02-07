use crate::UserID;
use anyhow::{anyhow, Context};
use judger::{OneOff, SourceFile, StoreFile, TaskReport};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use store::Handle;

use super::job_runner::JobRunner;

/// # Example
///
/// ```rust
#[doc = include_str!("../../examples/one_off.rs")]
/// ```
pub struct OneOffManager {
    base_dir: Handle,
    state: Arc<RwLock<HashMap<UserID, Result<TaskReport, String>>>>,
    runner: JobRunner,
}
impl OneOffManager {
    /// create `base_dir` if not exist
    ///
    /// spawn a new thread for job running
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(base_dir.as_ref())?;

        Ok(Self {
            base_dir: Handle::new(base_dir),
            state: Arc::new(RwLock::new(HashMap::new())),
            runner: JobRunner::new(),
        })
    }
    pub fn get_result(&self, uid: &UserID) -> anyhow::Result<Option<TaskReport>> {
        let guard = self
            .state
            .read()
            .map_err(|e| anyhow!("get read lock: {e}"))?;
        match guard.get(uid) {
            Some(r) => match r {
                Ok(r) => Ok(Some(r.clone())),
                Err(e) => Err(anyhow::Error::msg(e.clone())),
            },
            None => Ok(None),
        }
    }
    fn get_user_folder(&self, uid: &UserID) -> Handle {
        self.base_dir.join(uid.to_string())
    }
    pub fn add_test(
        &self,
        uid: UserID,
        source: SourceFile,
        input: StoreFile,
    ) -> anyhow::Result<()> {
        let base = self.get_user_folder(&uid);
        let state = self.state.clone();
        self.runner
            .add_job(move || {
                eprintln!("[job] oneoff uid = {uid}");
                state.write().unwrap().remove(&uid);
                std::fs::create_dir_all(&base).unwrap();
                #[cfg(target_os = "linux")]
                let mut one = OneOff::new(source, input, None);

                // 目前 macos 会出现无法杀死子进程导致评测失败的情况，尚未得到有效解决
                #[cfg(target_os = "macos")]
                let mut one = OneOff::new(
                    source,
                    input,
                    // TODO make configurable
                    Some("/Users/sshwy/zroj_core/target/debug/zroj-sandbox".into()),
                );
                one.set_wd(base);
                let result = one.exec().map_err(|e| e.to_string());
                eprintln!("[job] oneoff exec done.");
                // dbg!(&result);
                state.write().unwrap().insert(uid, result);
                eprintln!("[job] test done.")
            })
            .context("oneoff add job")
    }
}

impl Drop for OneOffManager {
    fn drop(&mut self) {
        tracing::info!("terminate oneoff manager");
    }
}
