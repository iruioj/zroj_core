use std::fmt::Debug;

use nix::errno::Errno;
use serde::{Deserialize, Serialize};

mod errno_serde {
    use nix::errno::Errno;
    use serde::{de::Visitor, Deserializer, Serializer};

    pub fn serialize<S>(errno: &Errno, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_i32(*errno as i32)
    }

    struct ErrnoVisitor;
    impl<'de> Visitor<'de> for ErrnoVisitor {
        type Value = Errno;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an i32")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Errno::from_i32(v))
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Errno, D::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_i32(ErrnoVisitor)
    }
}

/// 子进程（选手程序）在执行过程中遇到的评测错误
#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum ChildError {
    #[error("opening file: errno = {0}, mode = {1}")]
    OpenFile(#[serde(with = "errno_serde")] Errno, i32),
    #[error("redirect file: errno = {0}, to = {1}, from = {2}")]
    Dup(#[serde(with = "errno_serde")] Errno, i32, i32),
    #[error("setpgid: errno = {0}")]
    SetPGID(#[serde(with = "errno_serde")] Errno),
    #[error("setrlimit: errno = {0}, rsc = {1}, lim = {2}, {3}")]
    SetRlimit(#[serde(with = "errno_serde")] Errno, &'static str, u64, u64),
    #[error("execve: errno = {0}")]
    Execve(#[serde(with = "errno_serde")] Errno),
}

/// sandbox 运行时遇到的评测错误
#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum SandboxError {
    /// fork failed
    #[error("fork failed: {0}")]
    Fork(#[serde(with = "errno_serde")] Errno),
    /// can't be killed
    #[error("child process cannot be killed")]
    Unstoppable,
}
