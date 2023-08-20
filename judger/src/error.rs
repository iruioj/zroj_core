//! Error type.

use std::{fmt::Display, process::ExitStatus};

/// Judger error
#[derive(Debug)]
pub enum Error {
    CmdNotFound,
    /// 找到了命令，但是是一个 symlink
    CmdSymLink,
    Sandbox(sandbox::SandboxError),
    IOError(std::io::Error),
    CacheCE(sandbox::Status),
    SandboxExit(ExitStatus),
}

impl From<sandbox::SandboxError> for Error {
    fn from(value: sandbox::SandboxError) -> Self {
        Error::Sandbox(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CmdNotFound => write!(f, "Command not found"),
            Error::CmdSymLink => write!(f, "Command found but is symlink"),
            Error::Sandbox(e) => write!(f, "sandbox error: {}", e),
            Error::IOError(e) => write!(f, "io error: {}", e),
            Error::CacheCE(e) => write!(f, "compile error: {:?}", e),
            Error::SandboxExit(s) => write!(f, "sandbox exit error: {s}"),
        }
    }
}
impl std::error::Error for Error {}
