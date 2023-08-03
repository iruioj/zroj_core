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
    state: Arc<RwLock<HashMap<UserID, TaskReport>>>,
    queue: Arc<RwLock<Vec<Job>>>,
    sender: std::sync::mpsc::SyncSender<Job>,
    pub handle: std::thread::JoinHandle<()>,
}
impl OneOffManager {
    /// note: this will spawn a new thread for job running
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Self {
        std::fs::create_dir_all(base_dir.as_ref()).unwrap();
        // let job_lock = Mutex::new(());
        let queue = Arc::new(RwLock::new(Vec::<Job>::new()));
        let (sender, receiver) = std::sync::mpsc::sync_channel::<Job>(1);

        let que2 = queue.clone();
        let handle = std::thread::spawn(move || {
            eprintln!("[job] thread start");
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        eprintln!("[job] receive new job");
                        job();
                        while !que2.read().unwrap().is_empty() {
                            eprintln!("[job] consume job in queue");
                            let job = que2.write().unwrap().remove(0);
                            job();
                        }
                    }
                    Err(_) => {
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
        Ok(guard.get(uid).cloned())
    }
    pub fn get_user_folder(&self, uid: &UserID) -> Handle {
        self.base_dir.join(uid.to_string())
    }
    fn add_job<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let job = Box::new(f);
        if let Err(e) = self.sender.try_send(job) {
            match e {
                // 缓存已满
                std::sync::mpsc::TrySendError::Full(job) => {
                    self.queue
                        .write()
                        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
                        .push(job);
                }
                std::sync::mpsc::TrySendError::Disconnected(_) => {}
            }
        }
        Ok(())
    }
    pub fn add_test(&self, uid: UserID, source: StoreFile, input: StoreFile) -> Result<()> {
        let base = self.base_dir.clone();
        let state = self.state.clone();
        self.add_job(move || {
            eprintln!(
                "[job] test job uid = {uid}, source size = {}",
                source.file.metadata().unwrap().len()
            );
            {
                state.write().unwrap().remove(&uid);
            }
            let mut one = OneOff::new(source, input);
            one.set_wd(base);
            let result = one.exec().unwrap();
            eprintln!("[job] oneoff exec done.");
            dbg!(&result);
            state.write().unwrap().insert(uid, result);
            eprintln!("[job] test done.")
        })?;
        Ok(())
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
#include<unistd.h>
int main() {
    sleep(5);
    return 0;
}
",
            judger::FileType::GnuCpp17O2,
        );
        let input = StoreFile::from_str(r"1 2", judger::FileType::Plain);

        let h = std::thread::spawn(move || {
            oneoff.add_test(0, source, input).unwrap();

            oneoff.handle.join().unwrap();
        });
        h.join().unwrap();
        drop(dir)
    }
}
