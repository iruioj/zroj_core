/// WARNING: Cache Part UNTESTED!!!

use super::JudgeProblem;
use crate::{
    data::StoreFile,
    problem::{compile_in_wd, copy_in_wd},
    Checker, RuntimeError,
};
use judger::{
    sandbox::{mem, unix::SingletonBuilder, Builder, Elapse, ExecSandBox, Memory},
    truncstr::{TruncStr, TRUNCATE_LEN},
	cache::CompileResult,
};
use store::FsStore;

#[derive(FsStore, Debug)]
pub struct Meta {
    pub checker: Checker,
    // pub validator: String,
    /// 时间限制
    #[meta]
    pub time_limit: Elapse,
    /// 空间限制
    #[meta]
    pub memory_limit: Memory,
}

#[derive(FsStore, Debug)]
pub struct Task {
    pub input: StoreFile,
    pub output: StoreFile,
}

#[derive(FsStore, Debug)]
pub struct Subm {
    source: StoreFile,
}

/// 传统题（只是一个评测，数据直接用 ProblemStore 存）
pub struct Traditional;

impl JudgeProblem for Traditional {
    type T = Task;
    type M = Meta;
    type S = ();
    type Subm = Subm;

    // 先写了一个粗糙的，后面再来错误处理
    fn judge_task(
        &self,
        judger: &mut impl judger::Judger, // !!! TODO !!! 加 Mutex Lock
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> Result<judger::TaskReport, RuntimeError> {
        let wd = judger.working_dir();
        let Subm { source } = subm;

		eprintln!("编译源文件");

		match judger.compile(source) {
			None => {
				let term = compile_in_wd(source, &wd, "main")?;

				// Compile Error
				if !term.status.ok() {
					return Ok({
						let mut r = judger::TaskReport {
							status: judger::Status::CompileError(term.status),
							time: term.cpu_time.ms(),
							memory: term.memory.byte(),
							// todo: add log
							payload: Vec::new(),
						};
						let _ = r.add_payload("compile log", wd.join("main.c.log"));
						r
					});
				}
			},
			Some(c_res) => {
				match c_res.exec {
					None => {
						return Ok({
							let mut r = judger::TaskReport {
								status: judger::Status::CompileError(c_res.stat),
								// todo: add time, memory
								time: 0u64.into(),
								memory: 0u64.into(),
								// todo: add log
								payload: Vec::new(),
							};
							let _ = r.add_payload("compile log", wd.join("main.c.log"));
							r
						});
					},
					Some(exec) => {
						// !!! TODO !!! 处理文件错误
						let clog = c_res.clog;
						let mut m_exec = std::fs::File::create(wd.join("main")).unwrap();
						let mut m_clog = std::fs::File::create(wd.join("main.c.log")).unwrap();
						std::io::copy(&mut exec.open_file().unwrap(), &mut m_exec);
						std::io::copy(&mut clog.open_file().unwrap(), &mut m_clog);
					},
				}
			},
		};

        eprintln!("复制 IO 文件");
        copy_in_wd(&mut task.input, &wd, "input")?;
        copy_in_wd(&mut task.output, &wd, "answer")?;

        let s = SingletonBuilder::new(wd.join("main"))
            .stdin(wd.join("input"))
            .stdout(wd.join("output"))
            .stderr(wd.join("log"))
            .set_limits(|_| judger::sandbox::unix::Limitation {
                real_time: meta.time_limit.into(),
                cpu_time: meta.time_limit.into(),
                virtual_memory: meta.memory_limit.into(),
                real_memory: meta.memory_limit.into(),
                stack_memory: meta.memory_limit.into(),
                output_memory: mem!(64mb).into(),
                fileno: 5.into(),
            })
            .build()
            .unwrap();

        let term = s.exec_fork().unwrap();
        let term_status = term.status.clone();

        let mut report = judger::TaskReport::from(term);
        let _ = report.add_payload("compile log", wd.join("main.c.log"));
        let _ = report.add_payload("stdin", wd.join("input"));
        let _ = report.add_payload("stdout", wd.join("output"));
        let _ = report.add_payload("answer", wd.join("answer"));
        let _ = report.add_payload("stderr", wd.join("log"));

        if !term_status.ok() {
            return Ok(report);
        }

        // check answer
        let r = meta
            .checker
            .check(wd.join("input"), wd.join("output"), wd.join("answer"));

        if r.is_err() {
            report.status = judger::Status::WrongAnswer;
        }
        report.payload.push((
            "checker log".into(),
            TruncStr::new(
                match r {
                    Ok(s) => s,
                    Err(s) => s,
                },
                TRUNCATE_LEN,
            ),
        ));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use judger::{
        sandbox::{mem, time, Elapse, Memory},
        DefaultJudger,
    };
    use store::Handle;

    use crate::{data::StoreFile, problem::JudgeProblem, Checker};

    use super::{Meta, Subm, Task, Traditional};

    #[test]
    fn test_a_plus_b() {
        let a = Traditional;
        let dir = tempfile::tempdir().unwrap();
        let wd = Handle::new(dir);
        let mut jd = DefaultJudger::new(wd, None);
        let mut meta = Meta {
            checker: Checker::FileCmp,
            time_limit: time!(5s),
            memory_limit: mem!(256mb),
        };
        let mut task = Task {
            input: StoreFile::create_tmp("1 2"),
            output: StoreFile::create_tmp("3\n"),
        };
        let mut subm = Subm {
            source: {
                let mut r = StoreFile::create_tmp(
                    r#"#include<iostream>
                    using namespace std;
                    int main() {
                        int a, b;
                        cin >> a >> b;
                        cout << a + b << endl;
                    }
                    "#,
                );
                r.file_type = judger::FileType::GnuCpp14O2;
                r
            },
        };

        let report = a.judge_task(&mut jd, &mut meta, &mut task, &mut subm).unwrap();
        if let judger::Status::Accepted = report.status {
        } else {
            panic!("not accepted")
        }
        dbg!(report);
    }
}
