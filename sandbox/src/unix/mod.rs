pub(crate) mod singleton;
pub use singleton::Singleton;
pub use singleton::SingletonBuilder;
pub use singleton::Arg;

use serde::{Deserialize, Serialize};

/// 对进程施加各种类型的资源限制
#[derive(Serialize, Deserialize, Debug)]
pub struct Limitation {
    /// 限制实际运行时间，一般是用来做一个大保底
    pub real_time: Option<u64>,
    /// 限制 CPU 的运行时间，一般用来衡量程序的运行时间，单位：ms
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub cpu_time: Option<(u64, u64)>,
    /// 可以导致数组开大就会 MLE 的结果，单位：byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub virtual_memory: Option<(u64, u64)>,
    /// 程序执行完后才统计内存占用情况 （byte）
    pub real_memory: Option<u64>,
    /// byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub stack_memory: Option<(u64, u64)>,
    /// byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub output_memory: Option<(u64, u64)>,
    /// 限制文件指针数
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub fileno: Option<(u64, u64)>,
}