//! Error type.

use std::fmt::Display;

/// Judger error
#[derive(Debug)]
pub enum Error {
    CmdNotFound,
    /// 找到了命令，但是是一个 symlink
    CmdSymLink,
    Sandbox(sandbox::UniError),
    IOError(std::io::Error),
    CacheCE(sandbox::Status),
}

impl From<sandbox::UniError> for Error {
    fn from(value: sandbox::UniError) -> Self {
        Error::Sandbox(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CmdNotFound => write!(f, "Command not found"),
            Error::CmdSymLink => write!(f, "Command found but is symlink"),
            Error::Sandbox(e) => write!(f, "sandbox error: {}", e),
            Error::IOError(e) => write!(f, "io error: {}", e),
			Error::CacheCE(e) => write!(f, "compile error: {:?}", e),
        }
    }
}
impl std::error::Error for Error{}