use std::fmt::{Debug, Display};

/// sandbox 执行过程中产生的错误，俗称 system error
pub struct Error {
    msg: String,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("sandbox::Error")
            .field("msg", &self.msg)
            .finish()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sandbox Error: {}", self.msg)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error {
            msg: format!("stdio error: {}", value),
        }
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error {
            msg: format!("serde_json error: {}", value),
        }
    }
}
#[cfg(all(unix))]
impl From<nix::errno::Errno> for Error {
    fn from(value: nix::errno::Errno) -> Self {
        Error {
            msg: format!("waitpid failed, errno = {}", value)
        }
    }
}
impl From<std::ffi::FromBytesWithNulError> for Error {
    fn from(value: std::ffi::FromBytesWithNulError) -> Self {
        Error {
            msg: format!("bytes not end with null {}", value)
        }
    }
}
impl From<std::ffi::NulError> for Error {
    fn from(value: std::ffi::NulError) -> Self {
        Error {
            msg: format!("ffi nul error! {}", value)
        }
    }
}

/// return a Result error containing a message
pub fn msg_error<T>(msg: String) -> Result<T, Error> {
    Err(Error { msg: msg.clone() })
}