//! ZROJ 的评测模块
#![allow(dead_code)]

pub mod cache;
mod env;
mod error;
mod lang;
mod one_off;
mod report;
mod store_file;
pub mod truncstr;
pub use sha2;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

// pub use basic::Submission;
pub use cache::Cache;
pub use error::Error;
pub use lang::Compile;
pub use lang::FileType;
pub use one_off::OneOff;
pub use report::*;
pub use store::Handle;
pub use store_file::StoreFile;

/// Judger 是一个评测服务的上下文，可以提供评测环境的信息，访问相关缓存等等
///
/// Judger 不依赖于具体的题目类型，并且一般不会随题目评测完毕而销毁（持久化）
///
/// 写成 trait 的原因是 Judger 可以有不同的实现，例如跨平台实现、是否有缓存、是否实现了一些安全机制等等
pub trait Judger {
    /// 返回当前的工作目录
    fn working_dir(&self) -> store::Handle;
    /// 输出评测日志
    fn runtime_log(&mut self, msg: LogMessage);
}

pub struct DefaultJudger {
    wd: store::Handle,
}
impl DefaultJudger {
    pub fn new(wd: store::Handle) -> Self {
        Self { wd }
    }
}
impl Judger for DefaultJudger {
    fn working_dir(&self) -> store::Handle {
        self.wd.clone()
    }
    fn runtime_log(&mut self, msg: LogMessage) {
        eprintln!("[judger] {}", msg)
    }
}

/// 通过 channel 发送评测日志
pub struct MpscJudger {
    wd: store::Handle,
    sender: SyncSender<LogMessage>,
}

impl MpscJudger {
    pub fn new(wd: store::Handle) -> (Self, Receiver<LogMessage>) {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<LogMessage>(128);
        (Self { wd, sender }, receiver)
    }
}

impl Judger for MpscJudger {
    fn working_dir(&self) -> store::Handle {
        self.wd.clone()
    }

    fn runtime_log(&mut self, msg: LogMessage) {
        // ignore send error
        let _ = self.sender.send(msg);
    }
}

// use thiserror to conveniently define message content
#[derive(thiserror::Error, Debug, Clone)]
pub enum LogMessage {
    #[error("start judging (task kind: subtasks)")]
    StartSubtasks,
    #[error("start judging (task kind: tests)")]
    StartTests,
    #[error("judging subtask #{0} task #{1}")]
    SubtaskTask(usize, usize),
    #[error("judging task #{0}")]
    TestTask(usize),
    #[error("finished")]
    End,
}

pub mod sha_hash {
    pub use sha2::digest::Update; // re-export

    /// 可以转化为 SHA256 的哈希值（模仿 std::hash::Hash）
    pub trait ShaHash {
        fn sha_hash(&self, state: &mut sha2::Sha256);
    }

    impl ShaHash for String {
        fn sha_hash(&self, state: &mut sha2::Sha256) {
            state.update(self.as_bytes());
        }
    }

    impl ShaHash for &str {
        fn sha_hash(&self, state: &mut sha2::Sha256) {
            state.update(self.as_bytes());
        }
    }

    /// sequential hash: 将一系列（实现了 ShaHash）的对象哈希到一起。顺序会影响哈希值
    #[macro_export]
    macro_rules! seq_hash {
    [$( $e:expr ),*] => {
        {
            use $crate::sha2::{Sha256, Digest};
            use $crate::sha_hash::ShaHash;
			let mut hasher: Sha256 = Sha256::new();
			$( $e.sha_hash(&mut hasher); )*
			format!("{:x}", hasher.finalize())
        }
    };
}
}

// re-export
pub mod sandbox {
    pub use sandbox::*;
}
