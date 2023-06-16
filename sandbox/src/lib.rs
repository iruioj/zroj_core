#![warn(missing_docs)]
// #![feature(doc_auto_cfg)] // uncomment it to generate documents with platform-wide tag

//! Sandbox 库负责在限制的条件下执行可执行文件并返回执行的结果
//!
//! 为了避免繁琐的编译过程和开发环境搭建，本库将会基于 yaoj-judger 用 Rust 重写。

use serde::{Deserialize, Serialize};
use std::{
    ffi::{CString, NulError},
    fmt::Debug,
};

/// 沙盒运行过程中产生的错误（系统错误）
pub mod error;
pub use error::SandboxError;
/// Unix 系统下的沙盒 API
#[cfg(all(unix))]
pub mod unix;

#[cfg(all(windows))]
pub mod windows;

/// TLE 的具体类型
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TimeLimitExceededKind {
    /// 内核时间与用户时间之和
    Cpu,
    /// 进程的实际执行时间
    Real,
}

/// MLE 的具体类型
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MemoryLimitExceededKind {
    /// 虚拟内存
    Virtual,
    /// 实际使用内存（默认）
    Real,
    /// 栈空间
    Stack,
}

/// 执行的结果状态，只是一个初步的分析，适用于绝大多数情况
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Status {
    /// All Correct
    Ok,
    /// with error code and signal name
    RuntimeError(i32, Option<String>),
    /// 超出内存限制
    MemoryLimitExceeded(MemoryLimitExceededKind),
    /// 超出时间限制
    TimeLimitExceeded(TimeLimitExceededKind),
    /// 输出文件大小超出限制
    OutputLimitExceeded,
    /// 调用了被禁止的系统调用
    DangerousSyscall,
}

impl Status {
    /// if it is ok
    pub fn ok(&self) -> bool {
        if let Self::Ok = self {
            true
        } else {
            false
        }
    }
}

#[cfg(all(unix))]
impl From<nix::sys::signal::Signal> for Status {
    fn from(signal: nix::sys::signal::Signal) -> Self {
        Self::RuntimeError(0, Some(signal.to_string()))
    }
}

/// 终止时的信息
#[derive(Serialize, Deserialize, Debug)]
pub struct Termination {
    /// 终止状态
    pub status: Status,
    /// 实际运行时间 (ms)
    pub real_time: u64,
    /// CPU 占用时间 (ms)
    pub cpu_time: u64,
    /// 实际占用内存 (byte)
    pub memory: u64,
}
impl Termination {
    #[deprecated]
    fn _new() -> Self {
        Termination {
            status: Status::Ok,
            real_time: 0,
            cpu_time: 0,
            memory: 0,
        }
    }
}

#[cfg(all(unix))]
impl From<nix::sys::signal::Signal> for Termination {
    fn from(signal: nix::sys::signal::Signal) -> Self {
        // 存在优化的可能，即通过 signal 判断状态
        Self {
            status: Status::from(signal),
            real_time: 0,
            cpu_time: 0,
            memory: 0,
        }
    }
}

fn vec_str_to_vec_cstr(strs: &Vec<String>) -> Result<Vec<CString>, NulError> {
    strs.iter().map(|s| CString::new(s.clone())).collect()
}

/// 在沙箱中执行一系列的任务，返回相应的结果
pub trait ExecSandBox {
    /// 在实现时需要考虑 async-signal-safe，详见
    ///
    /// <https://docs.rs/nix/latest/nix/unistd/fn.fork.html#safety>
    ///
    fn exec_sandbox(&self) -> Result<Termination, SandboxError>;

    /// Unix Only: 在执行 exec_fork 内部执行此函数，如果失败会直接返回 Error，子进程会返回异常
    #[cfg(all(unix))]
    fn exec_sandbox_fork(&self, result_file: &mut std::fs::File) -> Result<(), SandboxError> {
        use std::io::Write;

        result_file.write(serde_json::to_string(&self.exec_sandbox()?)?.as_bytes())?;
        Ok(())
    }

    /// Unix only: 先 fork 一个子进程再执行程序，避免主进程终止导致整个进程终止
    #[cfg(all(unix))]
    fn exec_fork(&self) -> Result<Termination, SandboxError> {
        use crate::error::msg_err;
        use std::io::{Seek, SeekFrom};
        use tempfile::tempfile;

        use nix::sys::wait::{waitpid, WaitStatus};
        use nix::unistd::fork;
        use nix::unistd::ForkResult;

        let mut tmp = tempfile()?;

        match unsafe { fork() } {
            Err(_) => msg_err("fork failed"),
            Ok(ForkResult::Parent { child, .. }) => {
                match waitpid(child, None)? {
                    WaitStatus::Signaled(pid, signal, _) => {
                        msg_err(format!("主进程被杀死，pid = {}, signal = {}", pid, signal))
                    }
                    WaitStatus::Stopped(pid, signal) => {
                        msg_err(format!("主进程被停止，pid = {}, signal = {}", pid, signal))
                    }
                    WaitStatus::Exited(pid, code) => {
                        if code != 0 {
                            return msg_err(format!("主进程异常，code = {}，pid = {}", code, pid));
                        }
                        // 从开头读取
                        tmp.seek(SeekFrom::Start(0))?;
                        let termination = serde_json::from_reader(tmp)?;
                        Ok(termination)
                    }
                    _ => msg_err("未知的等待结果"),
                }
            }
            Ok(ForkResult::Child) => match self.exec_sandbox_fork(&mut tmp) {
                Ok(_) => unsafe { nix::libc::_exit(0) },
                Err(_) => unsafe { nix::libc::_exit(1) },
            },
        }
    }
}

/// Builder trait indicates something can be transform into an [`ExecSandBox`].
pub trait Builder {
    #[allow(missing_docs)]
    type Target: ExecSandBox;
    /// Consume self to build the target.
    fn build(self) -> Result<Self::Target, SandboxError>;
}
