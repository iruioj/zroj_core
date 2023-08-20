//! 题目的评测框架
//! 
//! 主要作用是安排任务的评测顺序，将任务的评测结果合并

use std::marker::PhantomData;

use judger::{
    sandbox::{Elapse, Memory},
    Status, TaskMeta, SCOER_EPS,
};

use crate::{data::{Rule, Data}, Override, RuntimeError};
use judger::{JudgeReport, SubtaskReport, LogMessage};
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
            rule: rule.clone(),
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


pub trait JudgeTask {
    type T: FsStore;
    type M: FsStore;
    type S: FsStore + Override<Self::M>;
    type Subm: FsStore;

    /// 单个测试点的评测
    ///
    /// 注意，源文件的编译、checker 的编译等等事情也会放在这里一起做。
    /// 从“多测试点评测”的概念上看，其最本质的写法就是对不同的测试点，把所有的流程都走一遍。
    /// 当然我们可以在实现的时候结合缓存系统来提高效率。
    fn judge_task(
        judger: &mut impl judger::Judger,
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> Result<judger::TaskReport, RuntimeError>;
}

/// 题目数据 + 评测
#[derive(FsStore)]
pub struct JudgeData<T, M, S, J>
where
    T: FsStore,
    M: FsStore + Clone,
    S: Override<M> + FsStore,
    J: JudgeTask<T = T, M = M, S = S>,
{
    /// 评测的数据
    data: Data<T, M, S>,
    /// marker，题目的评测过程
    judge_task: PhantomData<J>,
}

impl<T, M, S, J> JudgeData<T, M, S, J>
where
    T: FsStore,
    M: FsStore + Clone,
    S: FsStore + Override<M>,
    J: JudgeTask<T = T, M = M, S = S>,
{
    pub fn from_data(data: Data<T, M, S>) -> Self {
        Self {
            data,
            judge_task: PhantomData,
        }
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
    pub fn judge(
        &mut self,
        mut judger: impl judger::Judger,
        subm: &mut J::Subm,
    ) -> Result<judger::JudgeReport, RuntimeError> {
        Ok(match &mut self.data.tasks {
            crate::data::Taskset::Subtasks { subtasks, deps } => {
                judger.runtime_log(LogMessage::StartSubtasks);
                let mut summary = Summarizer::new(Rule::Sum);
                let mut reports: Vec<SubtaskReport> = Vec::new();
                for (id, sbt) in subtasks.iter_mut().enumerate() {
                    let dependency_ok = deps.iter().filter(|d| d.depender() == id).all(|d| {
                        matches!(reports[d.dependee()].meta.status, judger::Status::Good)
                    });

                    let mut subreports = Vec::new();
                    let mut sub_summary = Summarizer::new(Rule::Minimum);
                    for (tid, task) in sbt.tasks.iter_mut().enumerate() {
                        if !dependency_ok || sub_summary.skippable() || summary.skippable() {
                            // skip
                            subreports.push(None);
                        } else {
                            judger.runtime_log(LogMessage::SubtaskTask(id, tid));
                            judger.working_dir().remove_all()?;

                            let mut meta = self.data.meta.clone();
                            sbt.meta.over(&mut meta);
                            let r = J::judge_task(&mut judger, &mut meta, task, subm)?;

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
                        let r = J::judge_task(&mut judger, &mut self.data.meta, task, subm)?;
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
}