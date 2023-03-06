use std::fmt::{Debug, Display};

/// 沙盒运行过程中产生的错误
#[derive(Debug)]
pub struct SandboxError (String);

impl<'a> Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SandboxError(str) = self;
        write!(f, "Error: {}", str)
    }
}
impl std::error::Error for SandboxError{}

macro_rules! impl_err {
    ($( $t:ty )+) => {
        $(
            impl From<$t> for SandboxError {
                fn from(value: $t) -> Self {
                    SandboxError(format!("{:?}", value))
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
pub fn msg_err<'a, T, M: Into<String>>(msg: M) -> Result<T, SandboxError> {
    Err(SandboxError(msg.into()))
}
