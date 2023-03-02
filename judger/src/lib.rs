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

use std::path::PathBuf;

/// 一个测试点提交的可能的返回状态
pub enum Status {
    Accepted,
    CompileError,
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
pub struct TruncStr(String, i32);

/// 一个测试点的测试结果
pub struct Result {
    status: Status,
    msg: TruncStr,
    time: u64,
    memory: u64,
    stdin: TruncStr,
    stdout: TruncStr,
    stderr: TruncStr,
    answer: TruncStr,
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
trait Judge {

}

/// Hack 表示证伪选手代码的过程
trait Hack {
}