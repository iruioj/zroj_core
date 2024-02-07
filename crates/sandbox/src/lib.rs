#![warn(missing_docs)]
// #![feature(doc_auto_cfg)] // uncomment it to generate documents with platform-wide tag

//! Sandbox 库负责在限制的条件下执行可执行文件并返回执行的结果
//!
//! 为了避免繁琐的编译过程和开发环境搭建，本库将会基于 yaoj-judger 用 Rust 重写。

use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    time::Duration,
};

/// Unix 系统下的沙盒 API
#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

/// 执行的结果状态，只是一个初步的分析，适用于绝大多数情况
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, TsType)]
#[ts(name = "SandboxStatus")]
pub enum Status {
    /// All Correct
    Ok,
    /// with status code
    RuntimeError(i32),
    /// 超出内存限制
    MemoryLimitExceeded,
    /// 超出时间限制
    TimeLimitExceeded,
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

/// 在沙箱中执行一系列的任务，返回相应的结果
pub trait ExecSandBox {
    /// 在实现时需要考虑 async-signal-safe，详见
    ///
    /// <https://docs.rs/nix/latest/nix/unistd/fn.fork.html#safety>
    ///
    fn exec_sandbox(&self) -> anyhow::Result<Termination>;
}

/// 时间表示，数值单位为 ms
#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord, TsType,
)]
pub struct Elapse(u64);

impl From<Elapse> for u64 {
    fn from(value: Elapse) -> Self {
        value.0
    }
}

impl std::ops::Add<Elapse> for Elapse {
    type Output = Elapse;

    fn add(self, rhs: Elapse) -> Self::Output {
        Elapse(self.0 + rhs.0)
    }
}

impl Elapse {
    /// 输出以秒为单位的时间
    pub fn sec(self) -> u64 {
        self.0 / 1000
    }
    /// 输出以毫秒为单位的时间
    pub fn ms(self) -> u64 {
        self.0
    }
    /// pretty print
    pub fn pretty(self) -> String {
        format!("{self}ms")
    }
    /// create Elapse from seconds value
    pub fn from_sec(value: u64) -> Self {
        Self::from(value * 1000)
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

impl From<Memory> for u64 {
    fn from(value: Memory) -> Self {
        value.0
    }
}

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
    /// pretty print
    pub fn pretty(self) -> String {
        if self.0 < 1000 {
            format!("{self}bytes")
        } else if self.0 < 1_000_000 {
            format!("{}kb", (self.0 as f64) / 1024.0)
        } else {
            format!("{}mb", (self.0 as f64) / 1024.0 / 1024.0)
        }
    }
    /// create Memory size from MB value
    pub fn from_mb(value: u64) -> Self {
        Self::from(value << 20)
    }
}

/// copy from nix, create a null-terminate c-style string array.
/// This function is not necessarily async-signal safe
fn to_exec_array(args: Vec<std::ffi::CString>) -> Vec<*mut std::ffi::c_char> {
    use std::iter::once;
    args.into_iter()
        .map(|s| s.into_raw())
        .chain(once(std::ptr::null_mut()))
        .collect()
}