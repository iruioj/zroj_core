//! ZROJ 的评测模块
//!
//! To 高爸：
//!
//! 传统题的评测放在 crate::basic 中，
//! 交互题放在 crate::interact 中，
//! 提交答案题放在 crate::chkraw 中，
//! 通信题放在 crate::comms 中。
//!

mod basic;
mod env;
mod error;
mod one_off;

pub use error::Error;
pub use one_off::OneOff;
use serde::{Serialize, Deserialize};

/// 一个测试点提交的可能的返回状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Accepted,
    /// 编译失败，发生了无法处理的错误
    CompileFailed(sandbox::UniError),
    /// 编译错误，多半是选手问题
    CompileError(sandbox::Status),
    CustomMessage(String),
    DangerousSyscall,
    JudgementError,
    MemoryLimitExceeded,
    OutputLimitExceeded,
    PartiallyCorrect(f64),
    PresentationError,
    RuntimeError,
    TimeLimitExceeded,
    UnknownError,
    WrongAnswer,
}

/// 裁剪过的文本内容，包括省略的字节数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncStr(pub String, pub i32);

impl From<String> for TruncStr {
    fn from(value: String) -> Self {
        Self(value, 0)
    }
}
impl From<&str> for TruncStr {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

/// 一个测试点的测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResult {
    pub status: Status,
    pub msg: TruncStr,
    pub time: u64,
    pub memory: u64,
    // stdin: TruncStr,
    pub stdout: TruncStr,
    pub stderr: TruncStr,
    // answer: TruncStr,
}

pub mod lang {
    use std::path::PathBuf;

    pub trait LangOption {
        // 添加一个指令。比如说 O2，cpp17，nostd 等等的 flag
        // fn add_directive(&mut self, directive: String) -> &mut Self;

        /// 生成一个编译指令
        #[cfg(all(unix))]
        fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton;
    }

    /// 使用 g++ 编译 C++ 源文件
    pub struct GnuCpp {
        extra_args: Vec<String>,
    }

    impl GnuCpp {
        pub fn new(args: Vec<&'static str>) -> Self {
            let extra_args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
            GnuCpp { extra_args }
        }
    }

    impl LangOption for GnuCpp {
        fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton {
            let mut envs = Vec::new();
            for (key, value) in std::env::vars() {
                envs.push(format!("{}={}", key, value));
            }
            let envs = envs;
            // eprintln!("envs = {:?}", envs);

            let gpp = crate::env::which("g++").unwrap();
            sandbox::sigton! {
                exec: gpp;
                cmd: "g++" self.extra_args.clone() source "-o" dest;
                env: envs;
                lim cpu_time: 10000 10000; // 10s
                lim real_time: 10000;
                lim real_memory: 1024 * 1024 * 1024;
                lim virtual_memory: 1024 * 1024 * 1024 1024 * 1024 * 1024;
                lim stack: 1024 * 1024 * 1024 1024 * 1024 * 1024;
                lim output: 64 * 1024 * 1024 64 * 1024 * 1024;
                lim fileno: 50 50;
            }
        }
    }
    pub fn gnu_cpp20_o2() -> GnuCpp {
        GnuCpp::new(vec!["-std=c++2a", "-O2"])
    }
    pub fn gnu_cpp17_o2() -> GnuCpp {
        GnuCpp::new(vec!["-std=c++17", "-O2"])
    }
    pub fn gnu_cpp14_o2() -> GnuCpp {
        GnuCpp::new(vec!["-std=c++14", "-O2"])
    }
}
// 支持的语言
// pub enum LangOption {
//     // C with directives separated by ','
//     C(String),
//     // C++ with directives separated by ','
//     Cpp(String),
//     /// python 3
//     Python,
// }

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
