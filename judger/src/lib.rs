//! ZROJ 的评测模块
#![allow(dead_code)]

pub mod cache;
mod env;
mod error;
mod lang;
mod one_off;
mod report;
pub mod truncstr;

// pub use basic::Submission;
pub use cache::Cache;
pub use error::Error;
pub use lang::Compile;
pub use lang::FileType;
pub use one_off::OneOff;
pub use report::{JudgeReport, Status, TaskReport};

/// Judger 是一个评测服务的上下文，可以提供评测环境的信息，访问相关缓存等等
///
/// Judger 不依赖于具体的题目类型，并且一般不会随题目评测完毕而销毁（持久化）
///
/// 写成 trait 的原因是 Judger 可以有不同的实现，例如跨平台实现、是否有缓存、是否实现了一些安全机制等等
pub trait Judger {
    /// 返回当前的工作目录
    fn working_dir(&self) -> store::Handle;
}

pub struct DefaultJudger {
    wd: store::Handle,
}
impl DefaultJudger {
    pub fn new(wd: store::Handle) -> Self {
        Self { wd }
    }
}
impl Judger for DefaultJudger{
    fn working_dir(&self) -> store::Handle {
        self.wd.clone()
    }
}

/// Judge 表示的评测过程.
///
/// 考虑到评测的过程中很多文件可以再利用（比如选手的源文件，编译花费的时间很长，最好只编译一次，
/// 这种优化在比赛评测时很有效，也就是说终测的时候不用再次编译，这样会很节省评测时间），
/// 因此不把它写成一个 generic function 的形式，改用 trait 实现。大概会有
///
/// - pre_judge：检查参数是否正确，临时文件/目录初始化啥的。做一些编译啥的活，有的可以缓存
/// - judge：枚举测试点去评测（可能要考虑依赖），然后直接返回
/// - post_judge：收尾（删除临时文件啥的）
///
trait Judge {}

/// Hack 表示证伪选手代码的过程
trait Hack {}
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
            use sha2::{Sha256, Digest};
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
