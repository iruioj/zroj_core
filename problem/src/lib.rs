//! 题目的数据
//! 
//! 为什么要区分 Data 和 JudgeData：
//! 
//! - 题目本身在存储的时候是不用考虑提交记录的格式的
//! - 提交记录的处理只与题目的评测有关，不会影响题目的数据
//! - 同一个题目可能有不同的评测方式（一个常见的情况是将 stdio 的题目转化为文件 IO 供线下比赛评测）
mod checker;
pub mod data;
pub mod database;
mod error;
pub mod problem;
pub mod render_data;
pub mod prob_judger;

pub use checker::Checker;
pub use error::{DataError, RuntimeError};
pub use judger::sandbox::{Elapse, Memory};
use store::FsStore;

/// markdown AST，用于传递给前端
pub type Mdast = md::Node;

/// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
pub trait Override<T> {
    fn over(self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(self, _: &mut T) {}
}

/// 题目的数据、题面和题解组成的完整数据
#[derive(FsStore)]
pub struct ProblemFullData {
    data: problem::StandardProblem,
    #[meta]
    statement: render_data::Statement,
    #[meta]
    tutorial: render_data::Tutorial
}