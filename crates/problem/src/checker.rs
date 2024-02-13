use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use store::{FsStore, Handle};

fn compare_byline(
    output: BufReader<File>,
    answer: BufReader<File>,
    f: impl Fn(usize, String, String) -> Result<(), String>,
) -> Result<(), String> {
    let mut outs = output.lines().enumerate();
    let mut anss = answer.lines();

    loop {
        let out = outs.next();
        let ans = anss.next();
        if let Some((id, out)) = out {
            let out = out.map_err(|e| format!("read output line error: {e}"))?;
            if let Some(ans) = ans {
                let ans = ans.expect("read answer line error");
                f(id, out, ans)?
            } else {
                break Err("incorrect number of lines".into());
            }
        } else {
            break if ans.is_some() {
                Err("incorrect number of lines".into())
            } else {
                Ok(())
            };
        }
    }
}

/// OJ 内置的 Checker
///
/// 鉴于 testlib 年久失修并且非 rust 原生，输出格式不好控制，这里将常见的 checker 使用 rust 重写
#[derive(FsStore, Debug, Clone)]
pub enum Checker {
    /// 全文比较，忽略文末回车
    FileCmp,
    /// 对每行进行忽略空白字符的依次比较
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
    // Testlib {
    //     source: StoreFile,
    // },
}

fn file_cmp(fout: BufReader<File>, fans: BufReader<File>) -> Result<String, String> {
    compare_byline(fout, fans, |id, out, ans| {
        if out == ans {
            Ok(())
        } else {
            Err(format!("different at line {id}"))
        }
    })
    .map(|_| "correct.".into())
}
fn auto_cmp(
    fout: BufReader<File>,
    fans: BufReader<File>,
    abs_eps: f64,
    rel_eps: f64,
) -> Result<String, String> {
    compare_byline(fout, fans, |id, out, ans| {
        let out = out.split_whitespace();
        let mut ans = ans.split_whitespace();
        out.enumerate().try_fold((), |_, (tid, out)| {
            let Some(ans) = ans.next() else {
                return Err(format!("incorrect number of tokens at line {id}"));
            };
            if ans == out {
                Ok(())
            } else if let Ok(ans) = ans.parse::<f64>() {
                let Ok(out) = out.parse::<f64>() else {
                    return Err(format!("fail to parse float, {tid}-th tokens at line {id}"));
                };
                if (ans - out).abs() < abs_eps
                    || (ans - out).abs() / rel_eps.max(f64::max(ans, out)) < rel_eps
                {
                    Ok(())
                } else {
                    Err(format!("incorrect float, {tid}-th tokens at line {id}"))
                }
            } else {
                Err(format!("fail to match {tid}-th tokens at line {id}"))
            }
        })
    })
    .map(|_| "correct.".into())
}

impl Checker {
    /// 检查正确性，返回正确与否和详细信息
    pub fn check(
        &mut self,
        _input: &Handle,
        output: &Handle,
        answer: &Handle,
    ) -> anyhow::Result<(f64, String)> {
        let Ok(fout) = output.open_file() else {
            return Ok((0., "can not open output file".into()));
        };
        let fout = BufReader::new(fout);
        let fans = BufReader::new(answer.open_file().context("can not open answer file")?);

        match self {
            Checker::FileCmp => match file_cmp(fout, fans) {
                Ok(msg) => Ok((1., msg)),
                Err(msg) => Ok((0., msg)),
            },
            // Checker::Testlib { source: _ } => todo!(),
            Checker::AutoCmp {
                float_relative_eps,
                float_absoulte_eps,
            } => match auto_cmp(fout, fans, *float_absoulte_eps, *float_relative_eps) {
                Ok(msg) => Ok((1., msg)),
                Err(msg) => Ok((0., msg)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use judger::StoreFile;

    use super::*;

    #[test]
    fn test_compare_byline() {
        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4\n", judger::FileType::Plain).file),
            |_, _, _| Ok(()),
        )
        .unwrap();

        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4\n", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            |_, _, _| Ok(()),
        )
        .unwrap();

        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4\n ", judger::FileType::Plain).file),
            |_, out, ans| {
                dbg!(out);
                dbg!(ans);
                Ok(())
            },
        )
        .unwrap_err();

        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4\n ", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            |_, out, ans| {
                dbg!(out);
                dbg!(ans);
                Ok(())
            },
        )
        .unwrap_err();

        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4\n\n", judger::FileType::Plain).file),
            |_, out, ans| {
                dbg!(out);
                dbg!(ans);
                Ok(())
            },
        )
        .unwrap_err();

        compare_byline(
            BufReader::new(StoreFile::from_str("1 2\n3 4\n\n", judger::FileType::Plain).file),
            BufReader::new(StoreFile::from_str("1 2\n3 4", judger::FileType::Plain).file),
            |_, out, ans| {
                dbg!(out);
                dbg!(ans);
                Ok(())
            },
        )
        .unwrap_err();
    }
}
