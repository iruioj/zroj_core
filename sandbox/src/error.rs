use std::fmt::{Debug, Display};

use serde::{Serialize, Deserialize};

/// 一个通用的错误类型
#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum SandboxError {
    /// 基于信息的错误
    Msg(String),
}

impl Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SandboxError::Msg(str) = self;
        write!(f, "SandboxError: {}", str)
    }
}
impl std::error::Error for SandboxError{}

macro_rules! impl_err {
    ($( $t:ty )+) => {
        $(
            impl From<$t> for SandboxError {
                fn from(value: $t) -> Self {
                    SandboxError::Msg(format!("{:?}", value))
                }
            }
        )+
    };
}

impl_err!(
    serde_json::Error
    std::io::Error
    std::string::String
    std::sync::mpsc::SendError<()>
    std::ffi::NulError
);

#[cfg(all(unix))]
impl_err!(
    nix::errno::Errno
);

/// return a Result error containing a message
pub fn msg_err<T, M: Into<String>>(msg: M) -> Result<T, SandboxError> {
    Err(SandboxError::Msg(msg.into()))
}
