//! 题目的数据
//!
//! 为什么要区分 Data 和 JudgeData：
//!
//! - 题目本身在存储的时候是不用考虑提交记录的格式的
//! - 提交记录的处理只与题目的评测有关，不会影响题目的数据
//! - 同一个题目可能有不同的评测方式（一个常见的情况是将 stdio 的题目转化为文件 IO 供线下比赛评测）
mod checker;
pub mod data;
mod error;
pub mod judger_framework;
pub mod problem;
pub mod render_data;
pub mod sample;
mod utils;

pub use crate::problem::StandardProblem;
pub use checker::Checker;
pub use error::RuntimeError;
pub use judger::sandbox::{Elapse, Memory};
use store::FsStore;

/// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
pub trait Override<T> {
    fn over(&self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(&self, _: &mut T) {}
}

/// 题目的数据、题面和题解组成的完整数据
#[derive(FsStore)]
pub struct ProblemFullData {
    pub data: StandardProblem,
    #[meta]
    pub statement: render_data::Statement,
    #[meta]
    pub tutorial: render_data::Tutorial,
}
