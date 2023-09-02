use super::job_runner::JobRunner;
use crate::SubmID;
use actix_web::error;
use judger::{JudgeReport, LogMessage, MpscJudger};
use problem::{
    data::OJData,
    judger_framework::{self, JudgeTask},
    Override,
};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use store::{FsStore, Handle};

#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
pub struct FullJudgeReport {
    pub pre: Option<JudgeReport>,
    pub data: Option<JudgeReport>,
    pub extra: Option<JudgeReport>,
}

impl FullJudgeReport {
    fn update(&mut self, other: FullJudgeReport) {
        if let Some(pre) = other.pre {
            self.pre.replace(pre);
        }
        if let Some(data) = other.data {
            self.data.replace(data);
        }
        if let Some(extra) = other.extra {
            self.extra.replace(extra);
        }
    }
}

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
        }
    }
    pub fn get_logs(&self, sid: &SubmID) -> actix_web::Result<Option<Vec<LogMessage>>> {
        Ok(self
            .logs
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
            .get(sid)
            .cloned())
    }
    /// remove result from state after judging WITHOUT checking judge status
    pub fn remove_result(&self, sid: &SubmID) -> Option<(FullJudgeReport, Vec<LogMessage>)> {
        Some((
            self.state
                .write()
                .expect("remove result from state")
                .remove(sid)?
                .ok()?,
            self.logs
                .write()
                .expect("remove log from state")
                .remove(sid)?,
        ))
    }
    pub fn add_test<T, M, S, J>(
        &self,
        sid: SubmID,
        ojdata: OJData<T, M, S>,
        mut subm: J::Subm,
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

                // marking the end of judging, a trigger to move submission result
                // to database
                logs2
                    .write()
                    .expect("finish judging of submission")
                    .entry(sid)
                    .or_default()
                    .push(LogMessage::Done);
                drop(logs2);

                Ok(())
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
