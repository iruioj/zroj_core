#![warn(missing_docs)]
// #![feature(doc_auto_cfg)] // uncomment it to generate documents with platform-wide tag

//! Sandbox 库负责在限制的条件下执行可执行文件并返回执行的结果
//!
//! 为了避免繁琐的编译过程和开发环境搭建，本库将会基于 yaoj-judger 用 Rust 重写。

use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::{
    ffi::{CString, NulError},
    fmt::{Debug, Display},
    time::Duration, str::FromStr
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, TsType)]
pub enum TimeLimitExceededKind {
    /// 内核时间与用户时间之和
    Cpu(Elapse),
    /// 进程的实际执行时间
    Real,
}

/// MLE 的具体类型
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, TsType)]
pub enum MemoryLimitExceededKind {
    /// 虚拟内存
    Virtual,
    /// 实际使用内存（默认）
    Real(Memory),
    /// 栈空间
    Stack,
}

/// 执行的结果状态，只是一个初步的分析，适用于绝大多数情况
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, TsType)]
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
        matches!(self, Self::Ok)
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
    pub real_time: Elapse,
    /// CPU 占用时间 (ms)
    pub cpu_time: Elapse,
    /// 实际占用内存 (byte)
    pub memory: Memory,
}

#[cfg(all(unix))]
impl From<nix::sys::signal::Signal> for Termination {
    fn from(signal: nix::sys::signal::Signal) -> Self {
        // 存在优化的可能，即通过 signal 判断状态
        Self {
            status: Status::from(signal),
            real_time: Elapse::default(),
            cpu_time: Elapse::default(),
            memory: Memory::default(),
        }
    }
}

fn vec_str_to_vec_cstr(strs: &[String]) -> Result<Vec<CString>, NulError> {
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

        result_file
            .write_all(
                serde_json::to_string(&self.exec_sandbox())
                    .expect("error serializing execution result")
                    .as_bytes(),
            )
            .expect("error writing result to file");
        Ok(())
    }

    /// Unix only: 先 fork 一个子进程再执行程序，避免主进程终止导致整个进程终止
    #[cfg(all(unix))]
    fn exec_fork(&self) -> Result<Termination, SandboxError> {
        use std::io::{Seek, SeekFrom};
        use tempfile::tempfile;

        use nix::sys::wait::{waitpid, WaitStatus};
        use nix::unistd::fork;
        use nix::unistd::ForkResult;

        let mut tmp = tempfile().unwrap();

        match unsafe { fork() } {
            Err(e) => Err(SandboxError::Fork(e.to_string())),
            Ok(ForkResult::Parent { child, .. }) => {
                match waitpid(child, None).expect("wait pid failed") {
                    WaitStatus::Signaled(pid, signal, _) => {
                        panic!("主进程被杀死，pid = {}, signal = {}", pid, signal)
                    }
                    WaitStatus::Stopped(pid, signal) => {
                        panic!("主进程被停止，pid = {}, signal = {}", pid, signal)
                    }
                    WaitStatus::Exited(pid, code) => {
                        if code != 0 {
                            panic!("主进程异常，code = {}，pid = {}", code, pid);
                        }
                        // 从开头读取
                        tmp.seek(SeekFrom::Start(0))
                            .expect("error seek start from tmp file");
                        serde_json::from_reader(tmp).expect("error reading termination result")
                    }
                    _ => panic!("未知的等待结果"),
                }
            }
            Ok(ForkResult::Child) => match self.exec_sandbox_fork(&mut tmp) {
                Ok(_) => unsafe { nix::libc::_exit(0) },
                Err(_) => unsafe { nix::libc::_exit(1) },
            },
        }
    }
}

/// 时间表示，数值单位为 ms
#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord, TsType,
)]
pub struct Elapse(u64);

impl Elapse {
    /// 输出以秒为单位的时间
    pub fn sec(self) -> u64 {
        self.0 / 1000
    }
    /// 输出以毫秒为单位的时间
    pub fn ms(self) -> u64 {
        self.0
    }
}

impl Display for Elapse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Elapse {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<u64> for Elapse {
    /// 单位：ms
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<Duration> for Elapse {
    fn from(value: Duration) -> Self {
        Self(value.as_millis() as u64)
    }
}

/// 内存空间表示，数值单位为 byte
#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord, TsType,
)]
pub struct Memory(u64);

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Memory {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<u64> for Memory {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Memory {
    /// 输出以字节为单位的时间
    pub fn byte(self) -> u64 {
        self.0
    }
}

#[allow(unused_imports)]
#[macro_use]
extern crate sandbox_macro;
pub use sandbox_macro::mem;
pub use sandbox_macro::time;
