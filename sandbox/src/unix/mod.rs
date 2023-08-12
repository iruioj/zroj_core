pub(crate) mod singleton;
pub use singleton::Arg;
pub use singleton::Singleton;
pub use singleton::SingletonBuilder;

use serde::{Deserialize, Serialize};

use crate::Elapse;
use crate::Memory;

/// 对资源 T 类型的限制
#[derive(Serialize, Deserialize, Debug)]
pub enum Lim<T>
where
    T: PartialOrd,
{
    /// 无限制
    None,
    /// 一个硬限制
    Single(T),
    /// soft and hard
    Double(T, T),
}

impl<T: PartialOrd + Copy> Lim<T> {
    /// 判断是否超过限制
    ///
    /// true 表示没有超过限制
    pub fn check(&self, usage: &T) -> bool {
        match self {
            Lim::None => true,
            Lim::Single(l) => usage <= l,
            Lim::Double(s, _) => usage <= s,
        }
    }
    /// 判断是否超过硬限制
    ///
    /// true 表示没有超过限制
    pub fn check_hard(&self, usage: &T) -> bool {
        match self {
            Lim::None => true,
            Lim::Single(l) => usage <= l,
            Lim::Double(_, h) => usage <= h,
        }
    }
}
impl<T: PartialOrd + Copy> From<T> for Lim<T> {
    fn from(value: T) -> Self {
        Lim::Single(value)
    }
}

/// 对进程施加各种类型的资源限制
#[derive(Serialize, Deserialize, Debug)]
pub struct Limitation {
    /// 限制实际运行时间，一般是用来做一个大保底
    /// 最好赋予 soft limit 和 hard limit 不同的值，详见 singleton 的实现
    pub real_time: Lim<Elapse>,
    /// 限制 CPU 的运行时间，一般用来衡量程序的运行时间
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub cpu_time: Lim<Elapse>,
    /// 可以导致数组开大就会 MLE 的结果
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub virtual_memory: Lim<Memory>,
    /// 程序执行完后才统计内存占用情况
    pub real_memory: Lim<Memory>,
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub stack_memory: Lim<Memory>,
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub output_memory: Lim<Memory>,
    /// 限制文件指针数
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub fileno: Lim<u64>,
}

/// 考虑安全性的默认限制，简单来说时间限制 1 分钟，空间限制 1 GB，最多同时打开 100 个文件
impl Default for Limitation {
    fn default() -> Self {
        Self {
            real_time: Lim::Double(60000.into(), 120000.into()),
            cpu_time: Lim::Single(60000.into()),
            virtual_memory: Lim::Single((1 << 30).into()),
            real_memory: Lim::Single((1 << 30).into()),
            stack_memory: Lim::Single((1 << 30).into()),
            output_memory: Lim::Single((1 << 30).into()),
            fileno: Lim::Single(100),
        }
    }
}
