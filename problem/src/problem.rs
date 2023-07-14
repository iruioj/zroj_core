use std::marker::PhantomData;

use crate::{
    data::{Data, Rule, StoreFile},
    DataError, Override, RuntimeError,
};

use judger::{
    sandbox::{Elapse, Memory},
    JudgeReport, JudgeReportMeta, SubtaskReport,
};
use store::{FsStore, Handle};

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

pub fn update_status(
    meta: &mut JudgeReportMeta,
    default_score: f64,
    rule: &Rule,
    status: judger::Status,
    time: Elapse,
    memory: Memory,
) {
    meta.time = meta.time.max(time);
    meta.memory = meta.memory.max(memory);
    let cur_score = status.score_rate() * status.total_score().unwrap_or(default_score);
    if let Rule::Sum = rule {
        meta.score += cur_score;
    } else {
        meta.score = meta.score.min(cur_score);
    }
    meta.status.update(status);
}

/// 题目数据结构 + 评测
pub struct JudgeData<T, M, S, J>
where
    T: FsStore,
    M: FsStore,
    S: Override<M> + FsStore,
    J: JudgeTask<T = T, M = M, S = S>,
{
    pub data: Data<T, M, S>,
    judge_task: PhantomData<J>,
}

impl<T, M, S, J> JudgeData<T, M, S, J>
where
    T: FsStore,
    M: FsStore,
    S: FsStore + Override<M>,
    J: JudgeTask<T = T, M = M, S = S> + Default,
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
        let mut meta = JudgeReportMeta {
            score: match &self.data.rule {
                crate::data::Rule::Sum => 0.0,
                crate::data::Rule::Minimum => 1.0,
            },
            status: judger::Status::Accepted,
            time: 0.into(),
            memory: 0.into(),
        };
        Ok(match &mut self.data.tasks {
            crate::data::Taskset::Subtasks { subtasks, deps } => {
                let mut reports: Vec<SubtaskReport> = Vec::new();
                for (id, sbt) in subtasks.iter_mut().enumerate() {
                    let dependency_ok = deps
                        .iter()
                        .filter(|d| d.depender() == id)
                        .all(|d| matches!(reports[d.dependee()].status, judger::Status::Accepted));
                    let default_score = 1.0 / sbt.tasks.len() as f64;
                    let mut sub_meta = JudgeReportMeta {
                        score: 1.0,
                        status: judger::Status::Accepted,
                        time: 0.into(),
                        memory: 0.into(),
                    };
                    let mut subreports = Vec::new();
                    for task in &mut sbt.tasks {
                        if !dependency_ok
                            || (self.data.rule == Rule::Minimum && meta.score < judger::SCOER_EPS)
                        {
                            // skip
                            subreports.push(None);
                        } else {
                            let r = J::judge_task(&mut judger, &mut self.data.meta, task, subm)?;
                            update_status(
                                &mut sub_meta,
                                default_score,
                                &Rule::Minimum,
                                r.status.clone(),
                                r.time.clone(),
                                r.memory.clone(),
                            );
                            subreports.push(Some(r));
                        }
                    }
                    reports.push(SubtaskReport {
                        status: sub_meta.status.clone(),
                        time: sub_meta.time,
                        memory: sub_meta.memory,
                        tasks: subreports,
                    });

                    meta.status.update(sub_meta.status.clone());
                    meta.time = meta.time.max(sub_meta.time);
                    meta.memory = meta.memory.max(sub_meta.memory);
                    let cur_score = sub_meta.status.score_rate()
                        * sub_meta.status.total_score().unwrap_or(sbt.score);
                    meta.score = match self.data.rule {
                        Rule::Sum => meta.score + cur_score,
                        Rule::Minimum => meta.score.min(cur_score),
                    }
                }
                JudgeReport {
                    meta,
                    detail: judger::JudgeDetail::Subtask(reports),
                }
            }
            crate::data::Taskset::Tests { tasks } => {
                let default_score = 1.0 / tasks.len() as f64;
                let r: Result<_, RuntimeError> = tasks.iter_mut().try_fold(
                    (meta, Vec::new(), default_score),
                    |(mut meta, mut reports, default_score), task| {
                        if self.data.rule == Rule::Minimum && meta.score < judger::SCOER_EPS {
                            // skip
                            reports.push(None);
                        } else {
                            let r = J::judge_task(&mut judger, &mut self.data.meta, task, subm)?;
                            update_status(
                                &mut meta,
                                default_score,
                                &self.data.rule,
                                r.status.clone(),
                                r.time.clone(),
                                r.memory.clone(),
                            );
                            reports.push(Some(r));
                        }

                        Ok((meta, reports, default_score))
                    },
                );
                let (meta, reports, ..) = r?;
                judger::JudgeReport {
                    meta,
                    detail: judger::JudgeDetail::Tests(reports),
                }
            }
        })
    }
}

/// 自动编译文件，可执行文件名为 name，编译日志为 name.c.log
fn compile_in_wd(
    file: &mut StoreFile,
    wd: &Handle,
    name: impl AsRef<str>,
) -> Result<judger::sandbox::Termination, RuntimeError> {
    use judger::Compile;
    let src = wd.join(String::from(name.as_ref()) + file.file_type.ext());
    let exec = wd.join(name.as_ref());
    let clog = wd.join(String::from(name.as_ref()) + ".c.log");

    file.copy_all(&mut src.create_new_file().map_err(DataError::from)?)
        .unwrap();

    let term = file
        .file_type
        .compile_sandbox(&src, &exec, &clog)
        .exec_fork()
        .unwrap();
    Ok(term)
}
fn copy_in_wd(file: &mut StoreFile, wd: &Handle, name: impl AsRef<str>) -> Result<(), DataError> {
    let src = wd.join(name.as_ref());
    file.copy_all(&mut src.create_new_file()?).unwrap();
    Ok(())
}

pub mod traditional;
pub type TraditionalData = Data<traditional::Task, traditional::Meta, ()>;
