use std::fmt::{Debug, Display};

use serde::{Serialize, Deserialize};

/// 一个通用的错误类型
#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum Error {
    /// 基于信息的错误
    Msg(String),
}

impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Error::Msg(str) = self;
        write!(f, "Error: {}", str)
    }
}
impl std::error::Error for Error{}

macro_rules! impl_err {
    ($( $t:ty )+) => {
        $(
            impl From<$t> for Error {
                fn from(value: $t) -> Self {
                    Error::Msg(format!("{:?}", value))
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
pub fn msg_err<'a, T, M: Into<String>>(msg: M) -> Result<T, Error> {
    Err(Error::Msg(msg.into()))
}
