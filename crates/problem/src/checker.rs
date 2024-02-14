use std::{
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use judger::{
    sandbox::{unix::Singleton, ExecSandBox},
    Judger, SourceFile, StoreFile, COMPILE_LIM,
};
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
#[derive(FsStore, Debug)]
#[non_exhaustive]
pub enum Checker {
    /// 全文比较，忽略文末回车
    FileCmp,
    /// 对每行进行忽略空白字符的依次比较。对于不能精确匹配的情况，尝试进行浮点数比较.
    AutoCmp {
        /// 相对误差，要求 `|a - b| / max(|a|, |b|, eps) < eps`
        #[meta]
        float_relative_eps: f64,
        /// 绝对误差，要求 `|a - b| < eps`
        #[meta]
        float_absoulte_eps: f64,
        /// 在进行比较之前转换为小写，等价于忽略大小写.
        ///
        /// 'Lowercase' is defined according to the terms of the Unicode Derived Core Property Lowercase.
        #[meta]
        to_lower_case: bool,
    },
    /// We provide builtin support for [Codeforces Testlib](https://github.com/MikeMirzayanov/testlib)
    /// checker
    TestlibChecker {
        // do not load the huge header file into memory
        testlib_header: StoreFile,
        checker: SourceFile,
    },
    /// Testlib is rather heavy and sometimes unnecessary to build a checker. For a lightweight implementation,
    /// you may provide arbitrary source with arbitrary language, which exposed the C ABI as
    ///
    /// ```c
    #[doc = include_str!("./checker_c_abi.h")]
    /// ```
    CABI { source: SourceFile },
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
    to_lower_case: bool,
) -> Result<String, String> {
    compare_byline(fout, fans, |id, out, ans| {
        let out = out.split_whitespace();
        let mut ans = ans.split_whitespace();
        out.enumerate().try_fold((), |_, (tid, out)| {
            let tid = tid + 1;
            let Some(ans) = ans.next() else {
                return Err(format!("incorrect number of tokens at line {id}"));
            };
            if ans == out || to_lower_case && ans.to_lowercase() == out.to_lowercase() {
                Ok(())
            } else if let Ok(ans) = ans.parse::<f64>() {
                let Ok(out) = out.parse::<f64>() else {
                    return Err(format!(
                        "fail to parse float for the {tid}-th tokens at line {id}"
                    ));
                };
                let delta = (ans - out).abs();
                if delta < abs_eps && delta / rel_eps.max(f64::max(ans, out)) < rel_eps {
                    Ok(())
                } else {
                    Err(format!(
                        "incorrect float of the {tid}-th tokens at line {id}"
                    ))
                }
            } else {
                Err(format!("fail to match the {tid}-th tokens at line {id}"))
            }
        })
    })
    .map(|_| "correct.".into())
}

impl Checker {
    /// 检查正确性，返回正确与否和详细信息
    pub fn check<M: std::fmt::Display>(
        &mut self,
        judger: &impl Judger<M>,
        input: &Handle,
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
            Checker::AutoCmp {
                float_relative_eps,
                float_absoulte_eps,
                to_lower_case,
            } => match auto_cmp(
                fout,
                fans,
                *float_absoulte_eps,
                *float_relative_eps,
                *to_lower_case,
            ) {
                Ok(msg) => Ok((1., msg)),
                Err(msg) => Ok((0., msg)),
            },
            Checker::TestlibChecker {
                testlib_header,
                checker,
            } => {
                judger.copy_store_file(testlib_header, "testlib.h")?;
                let judger::Compilation {
                    termination,
                    execfile,
                    ..
                } = judger.cachable_block(
                    |judger, checker| judger.compile(checker, "checker-pre"),
                    checker,
                )?;

                let mut execfile =
                    execfile.with_context(|| format!("compile checker error: {termination:?}"))?;
                let checker = judger.copy_file(&mut execfile, "checker")?;
                let checker_log = judger.clear_dest("checker.log")?;

                let term = judger::sandbox::unix::Singleton::new(checker.path())
                    .push_arg([
                        CString::new("checker").unwrap(),
                        input.to_cstring(),
                        output.to_cstring(),
                        answer.to_cstring(),
                    ])
                    .stderr(checker_log.to_cstring())
                    .exec_sandbox()?;

                let checker_log =
                    std::fs::read_to_string(&checker_log).context("read checker log")?;

                match term.status {
                    judger::sandbox::Status::Ok => Ok((1., checker_log)),
                    judger::sandbox::Status::RuntimeError(s) => {
                        Ok((0., format!("(checker exit code = {s}) {checker_log}")))
                    }
                    t => Err(anyhow::anyhow!("checker error: {t:?}, {checker_log}")),
                }
            }
            Checker::CABI { source } => {
                let exec = match source.file_type {
                    judger::FileType::GnuCpp20O2 => {
                        compile_cabi_checker_cpp(judger, source, "--std=c++2a")
                    }
                    judger::FileType::GnuCpp17O2 => {
                        compile_cabi_checker_cpp(judger, source, "--std=c++17")
                    }
                    judger::FileType::GnuCpp14O2 => {
                        compile_cabi_checker_cpp(judger, source, "--std=c++14")
                    }
                    judger::FileType::Rust => compile_cabi_checker_rust(judger, source),
                    _ => unimplemented!(),
                }?;
                let checker_out = judger.clear_dest("checker_stdout")?;

                // use default limitation
                Singleton::new(exec.path())
                    .push_arg([
                        CString::new("checker").unwrap(),
                        judger.working_dir().to_cstring(),
                    ])
                    .with_current_env()
                    .stdout(checker_out.to_cstring())
                    .exec_sandbox()?;

                let check_output = std::fs::read_to_string(&checker_out)?;
                let score: f64 = check_output
                    .split("\n")
                    .last()
                    .unwrap()
                    .parse()
                    .context("parse score from checker outputs")?;

                Ok((score, check_output))
            }
        }
    }
}

fn compile_cabi_checker_cpp<M: std::fmt::Display>(
    judger: &impl Judger<M>,
    source: &mut SourceFile,
    stdflag: &str,
) -> anyhow::Result<Handle> {
    judger.create_source_file(include_str!("./checker_c_abi.h"), "checker_c_abi.h")?;
    let c_abi_main =
        judger.create_source_file(include_str!("./checker_c_abi.c"), "checker_c_abi.c")?;
    let cpp_impl_src = judger.create_source_file(&source.source, "checker.cpp")?;
    let main_obj = judger.clear_dest("main.o")?;
    let checker_obj = judger.clear_dest("checker.o")?;
    let exec = judger.clear_dest("checker")?;

    Singleton::new(&judger::which("cc").unwrap())
        .push_arg([
            CString::new("cc").unwrap(),
            c_abi_main.to_cstring(),
            CString::new("-o").unwrap(),
            main_obj.to_cstring(),
            CString::new("-c").unwrap(),
            CString::new("-O2").unwrap(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    Singleton::new(&judger::which("g++")?)
        .push_arg([
            CString::new("g++").unwrap(),
            cpp_impl_src.to_cstring(),
            CString::new("-o").unwrap(),
            checker_obj.to_cstring(),
            CString::new("-c").unwrap(),
            CString::new(stdflag).unwrap(),
            CString::new("-O2").unwrap(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    Singleton::new(&judger::which("cc")?)
        .push_arg([
            CString::new("cc").unwrap(),
            CString::new("-o").unwrap(),
            exec.to_cstring(),
            main_obj.to_cstring(),
            checker_obj.to_cstring(),
            CString::new("-O2").unwrap(),
            CString::new("-lstdc++").unwrap(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    assert!(exec.path().exists());

    Ok(exec)
}

fn compile_cabi_checker_rust<M: std::fmt::Display>(
    judger: &impl Judger<M>,
    source: &mut SourceFile,
) -> anyhow::Result<Handle> {
    judger.create_source_file(include_str!("./checker_c_abi.h"), "checker_c_abi.h")?;
    let c_abi_main =
        judger.create_source_file(include_str!("./checker_c_abi.c"), "checker_c_abi.c")?;
    let rust_impl_src = judger.create_source_file(&source.source, "checker.rs")?;
    let main_obj = judger.clear_dest("main.o")?;
    let checker_lib = judger.clear_dest("libchecker.a")?;
    let exec = judger.clear_dest("checker")?;

    Singleton::new(&judger::which("cc").unwrap())
        .push_arg([
            CString::new("cc").unwrap(),
            c_abi_main.to_cstring(),
            CString::new("-o").unwrap(),
            main_obj.to_cstring(),
            CString::new("-c").unwrap(),
            CString::new("-O2").unwrap(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    Singleton::new(&judger::which("rustc")?)
        .push_arg([
            CString::new("rustc").unwrap(),
            CString::new("--crate-type=staticlib").unwrap(),
            rust_impl_src.to_cstring(),
            CString::new("-o").unwrap(),
            checker_lib.to_cstring(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    Singleton::new(&judger::which("cc")?)
        .push_arg([
            CString::new("cc").unwrap(),
            CString::new("-o").unwrap(),
            exec.to_cstring(),
            main_obj.to_cstring(),
            checker_lib.to_cstring(),
            CString::new("-O2").unwrap(),
            CString::new("-lpthread").unwrap(),
            CString::new("-ldl").unwrap(),
        ])
        .with_current_env()
        .set_limits(|_| COMPILE_LIM)
        .exec_sandbox()?;

    assert!(exec.path().exists());

    Ok(exec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use judger::StoreFile;
    use std::io::BufReader;

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
