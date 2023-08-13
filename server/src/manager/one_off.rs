use crate::UserID;
use actix_web::{error, Result};
use judger::{OneOff, StoreFile, TaskReport};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use store::Handle;

type Job = Box<dyn FnOnce() + Send + Sync + 'static>;
// #[derive(Debug)]
pub struct OneOffManager {
    base_dir: Handle,
    state: Arc<RwLock<HashMap<UserID, Result<TaskReport, String>>>>,
    queue: Arc<RwLock<Vec<Job>>>,
    sender: std::sync::mpsc::SyncSender<Option<Job>>,
    pub handle: std::thread::JoinHandle<()>,
}
impl OneOffManager {
    /// note: this will spawn a new thread for job running
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Self {
        std::fs::create_dir_all(base_dir.as_ref()).unwrap();
        // let job_lock = Mutex::new(());
        let queue = Arc::new(RwLock::new(Vec::<Job>::new()));
        let (sender, receiver) = std::sync::mpsc::sync_channel::<Option<Job>>(1);

        let que2 = queue.clone();
        let handle = std::thread::spawn(move || {
            eprintln!("[job] thread start");
            loop {
                match receiver.recv() {
                    Ok(Some(job)) => {
                        eprintln!("[job] receive new job");
                        job();
                        while !que2.read().unwrap().is_empty() {
                            eprintln!("[job] consume job in queue");
                            let job = que2.write().unwrap().remove(0);
                            job();
                        }
                    }
                    Ok(None) | Err(_) => {
                        // closed
                        eprintln!("[job] close job thread");
                        return;
                    }
                }
            }
        });
        Self {
            base_dir: Handle::new(base_dir),
            state: Arc::new(RwLock::new(HashMap::new())),
            queue,
            sender,
            handle,
        }
    }
    pub fn get_result(&self, uid: &UserID) -> Result<Option<TaskReport>> {
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
    fn add_job<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let job = Box::new(f);
        if let Err(e) = self.sender.try_send(Some(job)) {
            match e {
                // 缓存已满
                std::sync::mpsc::TrySendError::Full(Some(job)) => {
                    self.queue
                        .write()
                        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
                        .push(job);
                }
                std::sync::mpsc::TrySendError::Full(None)
                | std::sync::mpsc::TrySendError::Disconnected(_) => {}
            }
        }
        Ok(())
    }
    pub fn add_test(&self, uid: UserID, source: StoreFile, input: StoreFile) -> Result<()> {
        let base = self.get_user_folder(&uid);
        let state = self.state.clone();
        self.add_job(move || {
            eprintln!(
                "[job] test job uid = {uid}, source size = {}",
                source.file.metadata().unwrap().len()
            );
            {
                state.write().unwrap().remove(&uid);
            }
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
            dbg!(&result);
            state.write().unwrap().insert(uid, result);
            eprintln!("[job] test done.")
        })?;
        Ok(())
    }
    pub fn terminate(self) {
        eprintln!("terminate oneoff manager");
        self.sender.send(None).unwrap();
        self.handle.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_off_manager() {
        let dir = tempfile::TempDir::new().unwrap();
        let oneoff = OneOffManager::new(dir.path());

        let source = StoreFile::from_str(
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
