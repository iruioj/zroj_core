//! 题目的评测框架
//!
//! 主要作用是安排任务的评测顺序，将任务的评测结果合并

use judger::{
    sandbox::{Elapse, Memory},
    Status, TaskMeta, SCOER_EPS,
};

use crate::{
    data::{Data, Rule},
    Override,
};
use judger::{JudgeReport, SubtaskReport};
use std::sync::mpsc;
use store::FsStore;

pub struct Summarizer {
    status: Status,
    time: Elapse,
    memory: Memory,
    score: f64,
    rule: Rule,
}

impl Summarizer {
    pub fn new(rule: Rule) -> Self {
        Self {
            status: Status::Good,
            time: 0.into(),
            memory: 0.into(),
            score: match rule {
                Rule::Sum => 0.0,
                Rule::Minimum => 1.0,
            },
            rule,
        }
    }
    pub fn update(&mut self, r: &TaskMeta, task_score: f64) {
        self.status.update(r.status.clone());
        self.time = self.time.max(r.time);
        self.memory = self.memory.max(r.memory);
        let score = r.score_rate * task_score;
        self.score = match self.rule {
            Rule::Sum => self.score + score,
            Rule::Minimum => self.score.min(score),
        }
    }
    pub fn skippable(&self) -> bool {
        if matches!(self.rule, Rule::Minimum) && self.score < SCOER_EPS {
            return true;
        }
        false
    }
    pub fn report(&self) -> TaskMeta {
        TaskMeta {
            score_rate: self.score,
            status: self.status.clone(),
            time: self.time,
            memory: self.memory,
        }
    }
}

pub trait JudgeTask
where
    for<'a> &'a Self::S: Override<Self::M>,
{
    type T: FsStore;
    type M: FsStore;
    type S: FsStore;
    // any owned data always passes a 'static lifetime bound
    type Subm: FsStore + Send + Sync + 'static;

    /// 单个测试点的评测
    ///
    /// 注意，源文件的编译、checker 的编译等等事情也会放在这里一起做。
    /// 从“多测试点评测”的概念上看，其最本质的写法就是对不同的测试点，把所有的流程都走一遍。
    /// 当然我们可以在实现的时候结合缓存系统来提高效率。
    fn judge_task(
        judger: &mut impl judger::Judger<LogMessage>,
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> anyhow::Result<judger::TaskReport>;
}

/// 通过 channel 发送评测日志
pub struct MpscJudger {
    wd: store::Handle,
    sender: mpsc::SyncSender<LogMessage>,
}

impl MpscJudger {
    pub fn new(wd: store::Handle) -> (Self, mpsc::Receiver<LogMessage>) {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<LogMessage>(128);
        (Self { wd, sender }, receiver)
    }
}

impl judger::Judger<LogMessage> for MpscJudger {
    fn working_dir(&self) -> store::Handle {
        self.wd.clone()
    }

    fn runtime_log(&mut self, msg: LogMessage) {
        // ignore send error
        let _ = self.sender.send(msg);
    }
}

/// use thiserror to conveniently define message content
#[derive(thiserror::Error, Debug, Clone)]
pub enum LogMessage {
    #[error("start judging (task kind: subtasks)")]
    StartSubtasks,
    #[error("start judging (task kind: tests)")]
    StartTests,
    #[error("judging subtask #{0} task #{1}")]
    SubtaskTask(usize, usize),
    #[error("judging task #{0}")]
    TestTask(usize),
    #[error("finished")]
    End,
}

/// 题目的评测
///
/// 结果中的 time 和 memory 为单个测试点的最大用时 / 内存
///
/// 在测试点模式中，每个测试点的总分默认是等分，否则是 checker 返回的总分
///
/// 返回的得分是单位化的（0-1之间）
///
/// 子任务中的测试点默认按照 Min 的策略记分
pub fn judge<T, M, S, J>(
    data: &mut Data<T, M, S>,
    judger: &mut impl judger::Judger<LogMessage>,
    subm: &mut J::Subm,
) -> anyhow::Result<judger::JudgeReport>
where
    T: FsStore + Send + Sync + 'static,
    M: FsStore + Clone + Send + Sync + 'static,
    S: FsStore + Send + Sync + 'static,
    for<'a> &'a S: Override<M>,
    J: JudgeTask<T = T, M = M, S = S>,
{
    Ok(match &mut data.tasks {
        crate::data::Taskset::Subtasks { subtasks, deps } => {
            judger.runtime_log(LogMessage::StartSubtasks);
            let mut summary = Summarizer::new(Rule::Sum);
            let mut reports: Vec<SubtaskReport> = Vec::new();
            for (id, sbt) in subtasks.iter_mut().enumerate() {
                let dependency_ok = deps
                    .iter()
                    .filter(|d| d.depender() == id)
                    .all(|d| matches!(reports[d.dependee()].meta.status, judger::Status::Good));

                let mut subreports = Vec::new();
                let mut sub_summary = Summarizer::new(Rule::Minimum);
                for (tid, task) in sbt.tasks.iter_mut().enumerate() {
                    if !dependency_ok || sub_summary.skippable() || summary.skippable() {
                        // skip
                        subreports.push(None);
                    } else {
                        judger.runtime_log(LogMessage::SubtaskTask(id, tid));
                        judger.working_dir().remove_all()?;

                        let mut meta = data.meta.clone();
                        sbt.meta.over(&mut meta);
                        let r = J::judge_task(judger, &mut meta, task, subm)?;

                        sub_summary.update(&r.meta, 1.0);
                        subreports.push(Some(r));
                    }
                }
                let sub_meta = sub_summary.report();
                summary.update(&sub_meta, sbt.score);
                reports.push(SubtaskReport {
                    total_score: sbt.score,
                    meta: sub_meta,
                    tasks: subreports,
                });
            }
            judger.runtime_log(LogMessage::End);
            JudgeReport {
                meta: summary.report(),
                detail: judger::JudgeDetail::Subtask(reports),
            }
        }
        crate::data::Taskset::Tests { tasks } => {
            judger.runtime_log(LogMessage::StartTests);
            let default_score = 1.0 / tasks.len() as f64;
            let mut reports = Vec::new();
            let mut summary = Summarizer::new(Rule::Sum);
            for (id, task) in tasks.iter_mut().enumerate() {
                if summary.skippable() {
                    reports.push(None)
                } else {
                    judger.runtime_log(LogMessage::TestTask(id));
                    judger.working_dir().remove_all()?;
                    let r = J::judge_task(judger, &mut data.meta, task, subm)?;
                    summary.update(&r.meta, default_score);
                    reports.push(Some(r));
                }
            }
            judger.runtime_log(LogMessage::End);
            JudgeReport {
                meta: summary.report(),
                detail: judger::JudgeDetail::Tests(reports),
            }
        }
    })
}
