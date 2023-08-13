use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

/// 子进程（选手程序）在执行过程中遇到的评测错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildError {
    /// 打开文件时出错
    OpenFile(String, PathBuf, i32),
    /// 文件重定向时出错
    Dup(String, i32, i32),
    /// setpgid error
    SetPGID(String),
    /// setrlimit error
    SetRlimit(String, String, u64, u64),
    /// execve error
    Execve(String, PathBuf, Vec<String>, Vec<String>),
}

impl Display for ChildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChildError::OpenFile(err, path, flag) => write!(
                f,
                "opening file (path = {}, flag = {}): {err}",
                path.display(),
                flag
            ),
            ChildError::Dup(err, to, from) => write!(f, "dup from {from} to {to}: {err}"),
            ChildError::SetPGID(err) => write!(f, "setpgid: {err}"),
            ChildError::SetRlimit(err, name, s, h) => {
                write!(f, "setrlimit for {name:?} (soft = {s}, hard = {h}): {err}")
            }
            ChildError::Execve(err, path, args, env) => write!(
                f,
                "execve (path = {}, args = {args:?}, envs = {env:?}): {err}",
                path.display()
            ),
        }
    }
}

/// sandbox 运行时遇到的评测错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxError {
    /// 子进程出错
    Child(ChildError),
    /// fork failed
    Fork(String),
    /// can't be killed
    Unstoppable,
    /// not encouraged
    Custom(String)
}

impl Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::Child(err) => write!(f, "child process: {err}"),
            SandboxError::Fork(e) => write!(f, "fork failed: {e}"),
            SandboxError::Custom(e) => write!(f, "custom: {e}"),
            SandboxError::Unstoppable => write!(f, "can not stop"),
        }
    }
}

impl std::error::Error for SandboxError {}

// macro_rules! impl_err {
//     ($( $t:ty )+) => {
//         $(
//             impl From<$t> for SandboxError {
//                 fn from(value: $t) -> Self {
//                     SandboxError::Msg(format!("{:?}", value))
//                 }
//             }
//         )+
//     };
// }

// impl_err!(
//     serde_json::Error
//     std::io::Error
//     std::string::String
//     std::sync::mpsc::SendError<()>
//     std::ffi::NulError
// );

// #[cfg(all(unix))]
// impl_err!(
//     nix::errno::Errno
// );

// /// return a Result error containing a message
// pub fn msg_err<T, M: Into<String>>(msg: M) -> Result<T, SandboxError> {
//     Err(SandboxError::Msg(msg.into()))
// }
