//! Judger 返回的结果，可以直接在前端显示的数据格式，
//! 打通从 judger 到前端传递数据的过程

use crate::{truncstr::TruncStr, Error};
use serde::{Deserialize, Serialize};

/// 一个测试点提交的可能的返回状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", content = "payload", rename_all = "snake_case")]
pub enum Status {
    /// 通过
    Accepted,
    /// 编译错误
    CompileError(sandbox::Status),
    /// 自定义的评测状态
    Custom(String),
    DangerousSyscall,
    MemoryLimitExceeded,
    OutputLimitExceeded,
    /// (获得的部分分，总分）
    Partial(f64, f64),
    /// 非空字符构成的字符串与答案匹配
    PresentationError,
    RuntimeError,
    TimeLimitExceeded,
    WrongAnswer,
}

/// 一个测试点的测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReport {
    /// 评测结果
    pub status: Status,
    /// 花费时间
    pub time: u64,
    /// 占用内存
    pub memory: u64,
    /// 相关载荷（stdin, stdout, answer ...)
    pub payload: Vec<(String, TruncStr)>,
}

impl TaskReport {
    /// 从 path 中读取文件内容作为 payload
    pub(crate) fn add_payload(
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
}

impl From<sandbox::Termination> for TaskReport {
    fn from(value: sandbox::Termination) -> Self {
        Self {
            status: match value.status {
                sandbox::Status::Ok => Status::Accepted,
                sandbox::Status::RuntimeError(_, _) => Status::RuntimeError,
                sandbox::Status::MemoryLimitExceeded(_) => Status::MemoryLimitExceeded,
                sandbox::Status::TimeLimitExceeded(_) => Status::TimeLimitExceeded,
                sandbox::Status::OutputLimitExceeded => Status::OutputLimitExceeded,
                sandbox::Status::DangerousSyscall => Status::DangerousSyscall,
            },
            time: value.cpu_time,
            memory: value.memory,
            payload: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskReport {
    pub status: Status,
    pub time: u64,
    pub memory: u64,
    pub tasks: Vec<TaskReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeDetail {
    Subtask(Vec<SubtaskReport>),
    Tests(Vec<TaskReport>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeReport {
    pub status: Status,
    pub time: u64,
    pub memory: u64,
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
            status: crate::Status::WrongAnswer,
            time: 114,
            memory: 514,
            detail: super::JudgeDetail::Subtask(vec![SubtaskReport {
                status: crate::Status::WrongAnswer,
                time: 114,
                memory: 514,
                tasks: vec![TaskReport {
                    status: crate::Status::Partial(1., 2.),
                    time: 114,
                    memory: 514,
                    payload: vec![
                        ("stdin".to_string(), "1 2".into()),
                        ("stdout".to_string(), "2".into()),
                        ("answer".to_string(), "3".into()),
                    ],
                }],
            }]),
        };
        eprintln!("{}", serde_json::to_string_pretty(&r).unwrap());
    }
}
