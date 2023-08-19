//! Judger 返回的结果，可以直接在前端显示的数据格式，
//! 打通从 judger 到前端传递数据的过程

use crate::{truncstr::TruncStr, Error};
use sandbox::{Elapse, Memory};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;

/// 一个测试点提交的可能的返回状态
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
#[serde(tag = "name", content = "payload", rename_all = "snake_case")]
#[ts(name = "JudgerStatus")]
pub enum Status {
    /// 目前没有问题。不等价于通过（得看得分是否等于总分）
    Good,
    /// 编译错误
    CompileError(Option<sandbox::Status>),
    /// 自定义的评测状态
    Custom(String),
    DangerousSyscall,
    MemoryLimitExceeded,
    OutputLimitExceeded,
    // (获得的部分分，总分）
    // Partial(f64, f64),
    /// 非空字符构成的字符串与答案匹配
    PresentationError,
    RuntimeError,
    TimeLimitExceeded,
    WrongAnswer,
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
            _ => 0.0
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
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
pub struct TaskReport {
    /// 指标
    pub meta: TaskMeta,
    /// 相关载荷（stdin, stdout, answer ...)
    pub payload: Vec<(String, TruncStr)>,
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
    /// 从 path 中读取文件内容作为 payload
    pub fn add_payload(
        &mut self,
        name: impl AsRef<str>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Error> {
        self.payload.push((
            name.as_ref().to_string(),
            std::fs::read_to_string(path)
                .map_err(Error::IOError)?
                .into(),
        ));
        Ok(())
    }
    pub fn try_add_payload(mut self, name: impl AsRef<str>, path: impl AsRef<std::path::Path>) -> Self {
        let _ = self.add_payload(name, path);
        self
    }
}

impl From<sandbox::Status> for Status {
    fn from(value: sandbox::Status) -> Self {
        match value {
            sandbox::Status::Ok => Status::Good,
            sandbox::Status::RuntimeError(_, _) => Status::RuntimeError,
            sandbox::Status::MemoryLimitExceeded(_) => Status::MemoryLimitExceeded,
            sandbox::Status::TimeLimitExceeded(_) => Status::TimeLimitExceeded,
            sandbox::Status::OutputLimitExceeded => Status::OutputLimitExceeded,
            sandbox::Status::DangerousSyscall => Status::DangerousSyscall,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskReport {
    /// 所有子任务的分数总和为 1
    pub total_score: f64,
    pub meta: TaskMeta,
    pub tasks: Vec<Option<TaskReport>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeDetail {
    Subtask(Vec<SubtaskReport>),
    Tests(Vec<Option<TaskReport>>),
}

pub const SCOER_EPS: f64 = 1e-5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeReport {
    pub meta: TaskMeta,
    pub detail: JudgeDetail,
}

// 一次 hack 的结果
// struct HackResult {}

#[cfg(test)]
mod tests {
    use crate::{JudgeReport, TaskReport};

    use super::SubtaskReport;

    #[test]
    fn test_judge_result_serde() {
        let r = JudgeReport {
            meta: crate::TaskMeta {
                score_rate: 0.0,
                status: crate::Status::WrongAnswer,
                time: 114.into(),
                memory: 514.into(),
            },
            detail: super::JudgeDetail::Subtask(vec![SubtaskReport {
                total_score: 1.0,
                meta: crate::TaskMeta {
                    score_rate: 0.0,
                    status: crate::Status::WrongAnswer,
                    time: 114.into(),
                    memory: 514.into(),
                },
                tasks: vec![Some(TaskReport {
                    meta: crate::TaskMeta {
                        score_rate: 0.5,
                        status: crate::Status::Good,
                        time: 114.into(),
                        memory: 514.into(),
                    },
                    payload: vec![
                        ("stdin".to_string(), "1 2".into()),
                        ("stdout".to_string(), "2".into()),
                        ("answer".to_string(), "3".into()),
                    ],
                })],
            }]),
        };
        eprintln!("{}", serde_json::to_string_pretty(&r).unwrap());
    }
}
