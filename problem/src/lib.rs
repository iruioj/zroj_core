pub mod data;
mod error;
pub mod problem;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

fn compare_byline(
    output: BufReader<File>,
    answer: BufReader<File>,
    f: impl Fn(usize, String, String) -> Result<(), String>,
) -> Result<(), String> {
    let outs = output.lines().map_while(Result::ok).enumerate();
    let mut anss = answer.lines().map_while(Result::ok);
    for (id, out) in outs {
        if let Some(ans) = anss.next() {
            f(id, out, ans)?
        } else {
            return Err("incorrect number of lines".into());
        }
    }
    Ok(())
}
/// OJ 内置的 Checker
///
/// 鉴于 testlib 年久失修并且非 rust 原生，输出格式不好控制，这里将常见的 checker 使用 rust 重写
#[derive(FsStore, Debug)]
pub enum Checker {
    /// 全文比较
    FileCmp,
    /// 自动进行忽略空白字符的依次比较
    ///
    /// - 如果是字符串，要求全文匹配
    /// - 如果是整数，要求全文匹配
    /// - 如果是浮点数，要求在精度范围内匹配
    AutoCmp {
        /// 相对误差，要求 `|a - b| / max(|a|, |b|, eps) < eps`
        #[meta]
        float_relative_eps: f64,
        /// 绝对误差，要求 `|a - b| < eps`
        #[meta]
        float_absoulte_eps: f64,
    },
    Testlib {
        source: StoreFile,
    },
}

impl Checker {
    /// 检查正确性，返回正确与否和详细信息
    fn check(&mut self, _input: Handle, output: Handle, answer: Handle) -> (bool, String) {
        let fout = BufReader::new(match output.open_file() {
            Ok(r) => r,
            Err(_) => return (false, "can not open output file".into()),
        });
        let fans = BufReader::new(answer.open_file().expect("can not open answer file"));

        match self {
            Checker::FileCmp => {
                let r = compare_byline(fout, fans, |id, out, ans| {
                    if out == ans {
                        Ok(())
                    } else {
                        Err(format!("different at line {id}"))
                    }
                });
                match r {
                    Ok(_) => (true, "correct.".to_string()),
                    Err(e) => (false, e),
                }
            }
            Checker::Testlib { source: _ } => todo!(),
            Checker::AutoCmp {
                float_relative_eps,
                float_absoulte_eps,
            } => {
                let r = compare_byline(fout, fans, |id, out, ans| {
                    let out = out.split_whitespace();
                    let mut ans = ans.split_whitespace();
                    out.enumerate().try_fold((), |_, (tid, out)| {
                        if let Some(ans) = ans.next() {
                            if ans == out {
                                Ok(())
                            } else if let Ok(ans) = ans.parse::<f64>() {
                                if let Ok(out) = out.parse::<f64>() {
                                    if (ans - out).abs() < *float_absoulte_eps
                                    || (ans - out).abs() / float_relative_eps.max(f64::max(ans, out)) < *float_relative_eps {
                                        Ok(())
                                    } else {
                                        Err(format!("incorrect float, the {tid}-th tokens at line {id}"))
                                    }
                                } else {
                                    Err(format!("fail to parse float, the {tid}-th tokens at line {id}"))
                                }
                            } else {
                                Err(format!("fail to match the {tid}-th tokens at line {id}"))
                            }
                        } else {
                            Err(format!("incorrect number of tokens at line {id}"))
                        }
                    })
                });
                match r {
                    Ok(_) => (true, "correct.".to_string()),
                    Err(e) => (false, e),
                }
            }
        }
    }
}
