use super::job_runner::JobRunner;
use crate::{data::types::FullJudgeReport, SubmID};
use anyhow::{anyhow, Context};
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

/// # Example
///
/// ```rust
#[doc = include_str!("../../examples/problem_judger.rs")]
/// ```
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
    pub fn new(base_dir: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(base_dir.as_ref())?;

        Ok(Self {
            base_dir: Handle::new(base_dir),
            state: Arc::new(RwLock::new(HashMap::new())),
            logs: Default::default(),
            runner: JobRunner::new(),
            channel: crossbeam_channel::unbounded(),
        })
    }
    pub fn reciver(&self) -> crossbeam_channel::Receiver<(SubmID, FullJudgeReport)> {
        self.channel.1.clone()
    }
    pub fn get_logs(&self, sid: &SubmID) -> anyhow::Result<Option<Vec<LogMessage>>> {
        Ok(self
            .logs
            .read()
            .map_err(|e| anyhow!("get read lock for logs: {e}"))?
            .get(sid)
            .cloned())
    }
    pub fn add_test<T, M, S, J>(
        &self,
        sid: SubmID,
        ojdata: OJData<T, M, S>,
        mut subm: J::Subm,
        // callback: impl FnOnce(Result<FullJudgeReport, String>) -> Result<(), String> + Send + Sync,
    ) -> anyhow::Result<()>
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
        self.runner.add_job(job).context("add job")
    }
}
