//! Judger 返回的结果
//!
//! 前端如何展示评测的子任务、测试点的详细信息？
//! 一般来说这种数据格式都会放在后端接口来定义。
//! 但是这样做不利于自定义题目的显示，为此我们需要打通从 judger 到前端
//! 传递数据的过程，于是我们直接让 judger 返回可以直接在前端显示的数据格式。

use crate::truncstr::TruncStr;
use serde_derive::{Deserialize, Serialize};

/// 一个测试点提交的可能的返回状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", content = "payload", rename_all="snake_case")]
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
pub struct TaskResult {
    /// 评测结果
    pub status: Status,
    /// 花费时间
    pub time: u64,
    /// 占用内存
    pub memory: u64,
    /// 相关载荷（stdin, stdout, answer ...)
    pub payload: Vec<(String, TruncStr)>,
}

impl From<sandbox::Termination> for TaskResult {
    fn from(value: sandbox::Termination) -> Self {
        return Self {
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
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskResult {
    pub status: Status,
    pub time: u64,
    pub memory: u64,
    pub tasks: Vec<TaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeDetail {
    Subtask(Vec<SubtaskResult>),
    Tests(Vec<TaskResult>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResult {
    pub status: Status,
    pub time: u64,
    pub memory: u64,
    pub detail: JudgeDetail,
}

// 一次 hack 的结果
// struct HackResult {}

#[cfg(test)]
mod tests {
    use crate::{JudgeResult, TaskResult};

    use super::SubtaskResult;

    #[test]
    fn test_judge_result_serde() {
        let r = JudgeResult {
            status: crate::Status::WrongAnswer,
            time: 114,
            memory: 514,
            detail: super::JudgeDetail::Subtask(vec![SubtaskResult {
                status: crate::Status::WrongAnswer,
                time: 114,
                memory: 514,
                tasks: vec![TaskResult {
                    status: crate::Status::Partial(1., 2.),
                    time: 114,
                    memory: 514,
                    payload: vec![
                        ("stdin".to_string(), "1 2".into()),
                        ("stdout".to_string(), "2".into()),
                        ("answer".to_string(), "3".into()),
                    ]
                }],
            }]),
        };
        eprintln!("{}",serde_json::to_string_pretty(&r).unwrap());
    }
}
