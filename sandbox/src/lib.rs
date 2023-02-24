//! Sandbox 库负责在限制的条件下执行可执行文件并返回执行的结果
//!
//! 为了避免繁琐的编译过程和开发环境搭建，本库将会基于 yaoj-judger 用 Rust 重写。

use serde_derive::{Deserialize, Serialize};
use std::{
    ffi::{CString, NulError},
    fmt::Debug,
};

pub mod error;

/// Unix 系统下的沙盒 API
#[cfg(all(unix))]
pub mod unix;

/// 对进程施加各种类型的资源限制
#[derive(Serialize, Deserialize, Debug)]
enum Limitation {
    /// 限制实际运行时间，一般是用来做一个大保底
    RealTime(u32),
    /// 限制 CPU 的运行时间，一般用来衡量程序的运行时间，单位：ms
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    CpuTime(u32, u32),
    /// 可以导致数组开大就会 MLE 的结果，单位：byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    VirtualMemory(u32, u32),
    /// 程序执行完后才统计内存占用情况 （byte）
    ActualMemory(u32),
    /// byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    StackMemory(u32, u32),
    /// byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    OutputMemory(u32, u32),
    /// 限制文件指针数
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    Fileno(u32, u32),
}

/// 在子进程正常退出的情况下，检查资源限制对结果的影响
///
/// 不包括 OLE
///
/// 如果发现不满足限制，返回对应的 status。
fn check_limit(term: &Termination, lim: &Limitation) -> Option<Status> {
    if let Limitation::RealTime(tl) = lim {
        if term.real_time > *tl as i64 {
            return Some(Status::TimeLimitExceeded(TimeLimitExceededKind::Real));
        }
    }
    if let Limitation::ActualMemory(ml) = lim {
        if term.memory > *ml as i64 {
            return Some(Status::MemoryLimitExceeded(MemoryLimitExceededKind::Real));
        }
    }
    None
}

/// TLE 的具体类型
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum TimeLimitExceededKind {
    /// 内核时间与用户时间之和
    Cpu,
    /// 进程的实际执行时间
    Real,
}

/// MLE 的具体类型
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MemoryLimitExceededKind {
    /// 虚拟内存
    Virtual,
    /// 实际使用内存（默认）
    Real,
    /// 栈空间
    Stack,
}

/// 执行的结果状态，只是一个初步的分析，适用于绝大多数情况
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Status {
    /// All Correct
    Ok,
    /// with error code and signal name
    RuntimeError(i32, Option<String>),
    MemoryLimitExceeded(MemoryLimitExceededKind),
    TimeLimitExceeded(TimeLimitExceededKind),
    OutputLimitExceeded,
    DangerousSyscall,
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
    status: Status,
    real_time: i64,
    cpu_time: i64,
    memory: i64,
}
impl Termination {
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
    strs.iter()
        .map(|s| CString::new((*s).clone()))
        .into_iter()
        .collect()
}

/// 在沙箱中执行一系列的任务，返回相应的结果
pub trait ExecSandBox {
    /// 在实现时需要考虑 async-signal-safe，详见
    ///
    /// <https://docs.rs/nix/latest/nix/unistd/fn.fork.html#safety>
    ///
    fn exec_sandbox(&self) -> Result<Termination, error::Error>;

    /// Unix Only: 在执行 exec_fork 内部执行此函数，如果失败会直接返回 Error，子进程会返回异常
    #[cfg(all(unix))]
    fn exec_sandbox_fork(&self, result_file: &mut std::fs::File) -> Result<(), error::Error> {
        use std::io::Write;

        result_file.write(serde_json::to_string(&self.exec_sandbox()?)?.as_bytes())?;
        Ok(())
    }

    /// Unix only: 先 fork 一个子进程再执行程序，避免主进程终止导致整个进程终止
    #[cfg(all(unix))]
    fn exec_fork(&self) -> Result<Termination, error::Error> {
        use crate::error::msg_error;
        use std::io::{Seek, SeekFrom};
        use tempfile::tempfile;

        use nix::sys::wait::{waitpid, WaitStatus};
        use nix::unistd::fork;
        use nix::unistd::ForkResult;

        let mut tmp = tempfile()?;

        match unsafe { fork() } {
            Err(_) => msg_error("fork failed".to_string()),
            Ok(ForkResult::Parent { child, .. }) => {
                match waitpid(child, None)? {
                    WaitStatus::Signaled(pid, signal, _) => {
                        msg_error(format!("主进程被杀死，pid = {}, signal = {}", pid, signal))
                    }
                    WaitStatus::Stopped(pid, signal) => {
                        msg_error(format!("主进程被停止，pid = {}, signal = {}", pid, signal))
                    }
                    WaitStatus::Exited(pid, code) => {
                        if code != 0 {
                            return msg_error(format!(
                                "主进程异常，code = {}，pid = {}",
                                code, pid
                            ));
                        }
                        // 从开头读取
                        tmp.seek(SeekFrom::Start(0))?;
                        let termination = serde_json::from_reader(tmp)?;
                        Ok(termination)
                    }
                    _ => msg_error("未知的等待结果".to_string()),
                }
            }
            Ok(ForkResult::Child) => match self.exec_sandbox_fork(&mut tmp) {
                Ok(_) => unsafe { nix::libc::_exit(0) },
                Err(_) => unsafe { nix::libc::_exit(1) },
            },
        }
    }
}
