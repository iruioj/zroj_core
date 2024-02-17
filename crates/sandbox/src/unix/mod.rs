pub(crate) mod share_mem;
pub(crate) mod sigsafe;
pub(crate) mod singleton;

use std::fmt::Display;
use std::str::FromStr;

pub use singleton::Singleton;

use serde::{Deserialize, Serialize};

use crate::Elapse;
use crate::Memory;

/// 对资源 T 类型的限制
#[derive(Serialize, Deserialize, Debug)]
pub enum Lim<T>
where
    T: PartialOrd,
{
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
            Lim::Single(l) => usage <= l,
            Lim::Double(s, _) => usage <= s,
        }
    }
}
impl<T: PartialOrd + Display> Display for Lim<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lim::Single(l) => write!(f, "{l},-"),
            Lim::Double(s, h) => write!(f, "{s},{h}"),
        }
    }
}
impl<T: PartialOrd + FromStr> FromStr for Lim<T> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a: Vec<&str> = s.split(',').collect();
        match a.as_slice() {
            [s, h] => {
                let Ok(s) = s.parse() else {
                    return Err("soft limit not parsed");
                };
                let Ok(h) = h.parse() else {
                    return Ok(Self::Single(s));
                };
                Ok(Self::Double(s, h))
            }
            _ => Err("invalid str"),
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
    /// 限制实际运行时间，linux 上一般是用来做一个大保底
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准。
    /// 可以考虑对 > soft limit <= hard limit 的程序进行自动重测
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

impl Display for Limitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}:{}:{}",
            self.real_time,
            self.cpu_time,
            self.virtual_memory,
            self.real_memory,
            self.stack_memory,
            self.output_memory,
            self.fileno
        )
    }
}

impl FromStr for Limitation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r: Vec<&str> = s.split(':').collect();
        match r.as_slice() {
            [rt, ct, vm, rm, sm, om, fo] => Ok(Self {
                real_time: rt.parse()?,
                cpu_time: ct.parse()?,
                virtual_memory: vm.parse()?,
                real_memory: rm.parse()?,
                stack_memory: sm.parse()?,
                output_memory: om.parse()?,
                fileno: fo.parse()?,
            }),
            _ => Err("invalid limitation format"),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let d = Limitation::default();
        let d2: Limitation = d.to_string().parse().unwrap();
        dbg!(d.to_string(), d2);
    }
}
