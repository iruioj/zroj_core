use super::job_runner::JobRunner;
use crate::{data::types::FullJudgeReport, SubmID};
use actix_web::error;
use judger::{LogMessage, MpscJudger};
use problem::{
    data::OJData,
    judger_framework::{self, JudgeTask},
    Override,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use store::{FsStore, Handle};

fn update_state_data(
    state: &mut HashMap<SubmID, Result<FullJudgeReport, String>>,
    sid: SubmID,
    rep: FullJudgeReport,
) {
    if let Some(item) = state.get_mut(&sid) {
        if let Ok(item) = item {
            item.update(rep)
        }
    } else {
        state.insert(sid, Ok(rep));
    }
}

pub struct ProblemJudger {
    base_dir: Handle,
    state: Arc<RwLock<HashMap<SubmID, Result<FullJudgeReport, String>>>>,
    logs: Arc<RwLock<HashMap<SubmID, Vec<LogMessage>>>>,
    runner: JobRunner,
    channel: (
        crossbeam_channel::Sender<(SubmID, FullJudgeReport)>,
        crossbeam_channel::Receiver<(SubmID, FullJudgeReport)>,
    ),
}
impl ProblemJudger {
    /// create `base_dir` if not exist
    ///
    /// spawn a new thread for job running
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Self {
        std::fs::create_dir_all(base_dir.as_ref()).expect("creating oneoff dir");

        Self {
            base_dir: Handle::new(base_dir),
            state: Arc::new(RwLock::new(HashMap::new())),
            logs: Default::default(),
            runner: JobRunner::new(),
            channel: crossbeam_channel::unbounded(),
        }
    }
    pub fn reciver(&self) -> crossbeam_channel::Receiver<(SubmID, FullJudgeReport)> {
        self.channel.1.clone()
    }
    pub fn get_logs(&self, sid: &SubmID) -> actix_web::Result<Option<Vec<LogMessage>>> {
        Ok(self
            .logs
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
            .get(sid)
            .cloned())
    }
    pub fn add_test<T, M, S, J>(
        &self,
        sid: SubmID,
        ojdata: OJData<T, M, S>,
        mut subm: J::Subm,
        // callback: impl FnOnce(Result<FullJudgeReport, String>) -> Result<(), String> + Send + Sync,
    ) -> actix_web::Result<()>
    where
        T: FsStore + Send + Sync + 'static,
        M: FsStore + Clone + Send + Sync + 'static,
        S: FsStore + Send + Sync + 'static,
        for<'a> &'a S: Override<M>,
        J: JudgeTask<T = T, M = M, S = S>,
    {
        let state = self.state.clone();
        let logs = self.logs.clone();
        let logs2 = self.logs.clone();
        let dir = self.base_dir.join(sid.to_string());
        let sender = self.channel.0.clone();

        let job = move || {
            state.write().expect("clear state").remove(&sid);
            let r = || -> Result<_, String> {
                dir.remove_all().map_err(|e| e.to_string())?;
                std::fs::create_dir_all(dir.path()).map_err(|e| e.to_string())?;
                let (mut judger, receiver) = MpscJudger::new(dir.clone());

                // TODO: pre and extra
                let (_, mut data, _) = ojdata.into_triple();

                // create a new thread for receiving messages
                let log_handle = std::thread::spawn(move || {
                    logs.write().expect("clear previous log").remove(&sid);
                    loop {
                        match receiver.recv() {
                            Ok(msg) => {
                                let mut g = logs.write().expect("write log");
                                let entry = g.entry(sid).or_default();
                                entry.push(msg);
                            }
                            Err(_) => return,
                        }
                    }
                });

                let data_report =
                    judger_framework::judge::<_, _, _, J>(&mut data, &mut judger, &mut subm)
                        .map_err(|e| e.to_string())?;
                update_state_data(
                    &mut state.write().expect("save data state"),
                    sid,
                    FullJudgeReport {
                        pre: None,
                        data: Some(data_report),
                        extra: None,
                    },
                );

                drop(judger); // indirectly close log handle
                log_handle.join().expect("log thread should finish");

                logs2
                    .write()
                    .expect("remove log from state")
                    .remove(&sid)
                    .expect("remove logs from state");

                let result = state
                    .write()
                    .expect("remove result from state")
                    .remove(&sid)
                    .expect("get result from state")?;

                sender.send((sid, result)).map_err(|e| e.to_string())
            }();
            if let Err(e) = r {
                state
                    .write()
                    .expect("write error state")
                    .insert(sid, Err(e));
            }
            eprintln!("[job] problem test done.");
        };
        self.runner
            .add_job(job)
            .map_err(error::ErrorInternalServerError)
    }
    pub fn terminate(self) {
        self.runner.terminate();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use problem::{
        prelude::Traditional,
        sample::{a_plus_b_data, a_plus_b_std},
        StandardProblem,
    };

    use super::*;
    #[test]
    fn test_lock() {
        let lock = super::RwLock::new(0);
        *lock.write().unwrap() = 5;
        *lock.try_write().unwrap() = 6;
    }

    #[test]
    fn test_problem_judger() {
        let dir = tempfile::TempDir::new().unwrap();
        let dir_handle = Handle::new(dir.path());
        let problem_judger = ProblemJudger::new(dir_handle);

        let StandardProblem::Traditional(ojdata) = a_plus_b_data();
        let subm = a_plus_b_std();

        problem_judger
            .add_test::<_, _, _, Traditional>(0, ojdata, subm)
            .unwrap();
        println!("test added");

        std::thread::sleep(Duration::from_secs(5));
        dbg!(problem_judger.logs.read().unwrap().get(&0));

        problem_judger.terminate();
        drop(dir)
    }
}
