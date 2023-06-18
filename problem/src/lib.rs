pub mod data;
mod error;
pub mod problem;

use std::io::{BufRead, BufReader};

use data::StoreFile;
pub use error::{DataError, RuntimeError};
use store::{FsStore, Handle};
// mod problem_set;

/// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
pub trait Override<T> {
    fn over(self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(self, _: &mut T) {}
}

/// OJ 内置的 Checker
///
/// 鉴于 testlib 年久失修并且非 rust 原生，输出格式不好控制，这里将常见的 checker 使用 rust 重写
#[derive(FsStore)]
pub enum Checker {
    /// 全文比较
    FileCmp,
    Testlib {
        source: StoreFile,
    },
}

impl Checker {
    /// 检查正确性，返回正确与否和详细信息
    fn check(&mut self, _input: Handle, output: Handle, answer: Handle) -> (bool, String) {
        match self {
            Checker::FileCmp => {
                let fout = BufReader::new(match output.open_file() {
                    Ok(r) => r,
                    Err(_) => return (false, "can not open output file".into()),
                });
                let fans = BufReader::new(answer.open_file().expect("can not open answer file"));

                let fout = fout.lines().filter_map(|l| l.ok());
                let fans = fans.lines().filter_map(|l| l.ok());
                let diff = fout
                    .zip(fans)
                    .enumerate()
                    .find(|(_, (out, ans))| out != ans);

                match diff {
                    Some((id, _)) => (false, format!("different at line {}", id)),
                    None => (true, format!("correct.")),
                }
            }
            Checker::Testlib { source: _ } => todo!(),
        }
    }
}
