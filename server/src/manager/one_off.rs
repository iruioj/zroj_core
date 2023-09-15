use crate::UserID;
use actix_web::error;
use judger::{OneOff, SourceFile, StoreFile, TaskReport};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use store::Handle;

use super::job_runner::JobRunner;

pub struct OneOffManager {
    base_dir: Handle,
    state: Arc<RwLock<HashMap<UserID, Result<TaskReport, String>>>>,
    runner: JobRunner,
}
impl OneOffManager {
    /// create `base_dir` if not exist
    ///
    /// spawn a new thread for job running
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Self {
        std::fs::create_dir_all(base_dir.as_ref()).expect("creating oneoff dir");

        Self {
            base_dir: Handle::new(base_dir),
            state: Arc::new(RwLock::new(HashMap::new())),
            runner: JobRunner::new(),
        }
    }
    pub fn get_result(&self, uid: &UserID) -> actix_web::Result<Option<TaskReport>> {
        let guard = self
            .state
            .read()
            .map_err(|_| error::ErrorInternalServerError("Poisoned lock"))?;
        match guard.get(uid) {
            Some(r) => match r {
                Ok(r) => Ok(Some(r.clone())),
                Err(e) => Err(error::ErrorInternalServerError(e.to_string())),
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
    ) -> actix_web::Result<()> {
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
            .map_err(error::ErrorInternalServerError)
    }
    /// wait for job thread to finish and then close it
    pub fn terminate(self) {
        eprintln!("terminate oneoff manager");
        self.runner.terminate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_off_manager() {
        let dir = tempfile::TempDir::new().unwrap();
        let oneoff = OneOffManager::new(dir.path());

        let source = SourceFile::from_str(
            r"
#include<iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
",
            judger::FileType::GnuCpp17O2,
        );
        let input = StoreFile::from_str(r"1 2", judger::FileType::Plain);

        let h = std::thread::spawn(move || {
            oneoff.add_test(0, source, input).unwrap();
            oneoff.terminate();
        });
        h.join().unwrap();
        drop(dir)
    }
}
