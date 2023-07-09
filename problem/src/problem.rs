use std::marker::PhantomData;

use crate::{
    data::{Data, Rule, StoreFile},
    DataError, Override, RuntimeError,
};

use judger::sandbox::{Elapse, Memory};
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
    pub fn judge(
        &mut self,
        mut judger: impl judger::Judger,
        subm: &mut J::Subm,
    ) -> Result<judger::JudgeReport, RuntimeError> {
        let judge =
            |(mut score, mut status, mut time, mut memory, mut reports, default_score): (
                f64,
                judger::Status,
                Elapse,
                Memory,
                Vec<Option<judger::TaskReport>>,
                f64,
            ),
             task| {
                if score < judger::SCOER_EPS {
                    // skip
                    reports.push(None);
                } else {
                    let r = J::judge_task(&mut judger, &mut self.data.meta, task, subm)?;

                    status.update(r.status.clone());
                    time = if r.time > time { r.time } else { time };
                    memory = if r.memory > memory { r.memory } else { memory };
                    let cur_score =
                        r.status.score_rate() * r.status.total_score().unwrap_or(default_score);
                    if let Rule::Sum = self.data.rule {
                        score += cur_score;
                    } else {
                        score = score.min(cur_score);
                    }
                    reports.push(Some(r));
                }

                Ok((score, status, time, memory, reports, default_score))
            };
        Ok(match &mut self.data.tasks {
            crate::data::Taskset::Subtasks { subtasks, deps } => {
                todo!()
            }
            crate::data::Taskset::Tests { tasks } => {
                let default_score = 1.0 / tasks.len() as f64;
                let r: Result<_, RuntimeError> = tasks.iter_mut().try_fold(
                    (
                        match &self.data.rule {
                            crate::data::Rule::Sum => 0.0,
                            crate::data::Rule::Minimum => 1.0,
                        },
                        judger::Status::Accepted,
                        0.into(),
                        0.into(),
                        Vec::new(),
                        default_score,
                    ),
                    judge,
                );
                let (score, status, time, memory, reports, ..) = r?;
                judger::JudgeReport {
                    score,
                    status,
                    time,
                    memory,
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
