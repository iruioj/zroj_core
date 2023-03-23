//! Error type.

use std::fmt::Display;

/// Judger error
#[derive(Debug)]
pub enum Error {
    CmdNotFound,
    /// 找到了命令，但是是一个 symlink
    CmdSymLink,
    Sandbox(sandbox::UniError),
}

impl From<sandbox::UniError> for Error {
    fn from(value: sandbox::UniError) -> Self {
        Error::Sandbox(value)
    }
}


impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CmdNotFound => write!(f, "Command not found"),
            Error::CmdSymLink => write!(f, "Command found but is symlink"),
            Error::Sandbox(e) => write!(f, "sandbox error: {}", e),
        }
    }
}
impl std::error::Error for Error{}