mod checker;
pub mod data;
mod error;
pub mod problem;
// mod problem_set;

pub use checker::Checker;
pub use error::{DataError, RuntimeError};

/// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
pub trait Override<T> {
    fn over(self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(self, _: &mut T) {}
}
