mod checker;
pub mod data;
pub mod database;
mod error;
pub mod problem;
pub mod render_data;

pub use checker::Checker;
pub use error::{DataError, RuntimeError};
pub use judger::sandbox::{Elapse, Memory};

/// markdown AST，用于传递给前端
pub type Mdast = md::Node;

/// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
pub trait Override<T> {
    fn over(self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(self, _: &mut T) {}
}