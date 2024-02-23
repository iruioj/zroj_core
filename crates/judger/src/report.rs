//! Judger 返回的结果，可以直接在前端显示的数据格式，
//! 打通从 judger 到前端传递数据的过程

use crate::truncstr::TruncStr;
use anyhow::Context;
use sandbox::{Elapse, Memory};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;

/// 一个测试点提交的可能的返回状态
#[derive(Debug, Clone, Serialize, Deserialize, TsType, PartialEq, Eq)]
#[serde(tag = "name", content = "payload", rename_all = "snake_case")]
#[ts(name = "JudgerStatus")]
#[non_exhaustive]
pub enum Status {
    /// 目前没有问题。不等价于通过（得看得分是否等于总分）
    Good,

    /// 编译错误
    CompileError(Option<sandbox::Status>),

    // DangerousSyscall,
    /// 超出内存限制
    MemoryLimitExceeded,
    // OutputLimitExceeded,
    // (获得的部分分，总分）
    // Partial(f64, f64),
    // 非空字符构成的字符串与答案匹配
    // PresentationError,
    RuntimeError,
    TimeLimitExceeded,
    // WrongAnswer,
}

impl Status {
    pub fn update(&mut self, s: Status) {
        match s {
            Status::Good => {} // do nothing
            _ => {
                // 默认直接赋值，不考虑 self 本身
                *self = s;
            }
        }
    }
    pub fn direct_score_rate(&self) -> f64 {
        match self {
            Status::Good => 1.0,
            _ => 0.0,
        }
    }
}

/// 一个测试点的测试结果指标
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
pub struct TaskMeta {
    /// 得分率
    pub score_rate: f64,
    /// 评测结果
    pub status: Status,
    /// 花费时间
    pub time: Elapse,
    /// 占用内存
    pub memory: Memory,
}

impl TaskMeta {
    pub fn error_status(status: Status) -> Self {
        Self {
            score_rate: 0.0,
            status,
            time: 0.into(),
            memory: 0.into(),
        }
    }
}

/// 一个测试点的测试结果
#[derive(Clone, Serialize, Deserialize, TsType)]
pub struct TaskReport {
    /// 指标
    pub meta: TaskMeta,
    /// 相关载荷（stdin, stdout, answer ...)
    pub payload: Vec<(String, TruncStr)>,
}

impl std::fmt::Debug for TaskReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alternate = f.alternate();
        let mut builder = f.debug_struct("Task");
        builder
            .field("score_rate", &self.meta.score_rate)
            .field("status", &self.meta.status)
            .field("time", &self.meta.time)
            .field("memory", &self.meta.memory);
        for (name, content) in &self.payload {
            if alternate {
                let srcfile = crate::SourceFile {
                    source: content.to_string(),
                    file_type: crate::FileType::Plain,
                };
                builder.field(name, &srcfile);
            } else {
                builder.field(name, &content.as_str());
            }
        }
        builder.finish()
    }
}

impl TaskReport {
    pub fn new(meta: TaskMeta) -> Self {
        Self {
            meta,
            payload: Vec::new(),
        }
    }
}

impl TaskReport {
    pub fn add_payload_str(
        &mut self,
        name: impl AsRef<str>,
        content: String,
    ) -> anyhow::Result<()> {
        self.payload
            .push((name.as_ref().to_string(), content.into()));
        Ok(())
    }

    /// 从 path 中读取文件内容作为 payload
    pub fn add_payload(
        &mut self,
        name: impl AsRef<str>,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<()> {
        self.payload.push((
            name.as_ref().to_string(),
            std::fs::read_to_string(path)
                .context("add payload to task report")?
                .into(),
        ));
        Ok(())
    }
    pub fn try_add_payload(
        mut self,
        name: impl AsRef<str>,
        path: impl AsRef<std::path::Path>,
    ) -> Self {
        let _ = self.add_payload(name, path);
        self
    }
}

impl From<sandbox::Status> for Status {
    fn from(value: sandbox::Status) -> Self {
        match value {
            sandbox::Status::Ok => Status::Good,
            sandbox::Status::RuntimeError(_) => Status::RuntimeError,
            sandbox::Status::MemoryLimitExceeded => Status::MemoryLimitExceeded,
            sandbox::Status::TimeLimitExceeded => Status::TimeLimitExceeded,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TsType)]
pub struct SubtaskReport {
    /// 所有子任务的分数总和为 1
    pub total_score: f64,
    pub meta: TaskMeta,
    pub tasks: Vec<Option<TaskReport>>,
}

impl std::fmt::Debug for SubtaskReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("SubtaskReport");
        builder
            .field("total_score", &self.total_score)
            .field("meta", &self.meta);
        for (tid, task) in self.tasks.iter().enumerate() {
            if let Some(task) = task {
                builder.field(&format!("#{tid}"), &task);
            } else {
                builder.field(&format!("#{tid}"), &"skipped");
            }
        }

        builder.finish()
    }
}

#[derive(Clone, Serialize, Deserialize, TsType)]
#[serde(tag = "type", content = "tasks")]
pub enum JudgeDetail {
    Subtask(Vec<SubtaskReport>),
    Tests(Vec<Option<TaskReport>>),
}

impl std::fmt::Debug for JudgeDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subtask(subtasks) => {
                let mut builder = f.debug_struct("Subtasks");
                for (tid, task) in subtasks.iter().enumerate() {
                    builder.field(&format!("#{tid}"), &task);
                }
                builder.finish()
            }
            Self::Tests(tasks) => {
                let mut builder = f.debug_struct("Tasks");
                for (tid, task) in tasks.iter().enumerate() {
                    if let Some(task) = task {
                        builder.field(&format!("#{tid}"), &task);
                    } else {
                        builder.field(&format!("#{tid}"), &"skipped");
                    }
                }
                builder.finish()
            }
        }
    }
}

pub const SCOER_EPS: f64 = 1e-5;

#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
pub struct JudgeReport {
    pub meta: TaskMeta,
    pub detail: JudgeDetail,
}

// 一次 hack 的结果
// struct HackResult {}
