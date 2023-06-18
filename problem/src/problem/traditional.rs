use super::JudgeProblem;
use crate::{
    data::StoreFile,
    problem::{compile_in_wd, copy_in_wd},
    Checker, RuntimeError,
};
use judger::{
    sandbox::{unix::SingletonBuilder, Builder, ExecSandBox},
    truncstr::{TruncStr, TRUNCATE_LEN},
};
use store::FsStore;

#[derive(FsStore)]
pub struct Meta {
    pub checker: Checker,
    // pub validator: String,
    /// 时间限制
    #[meta]
    pub time_limit: u64,
    /// 空间限制
    #[meta]
    pub memory_limit: u64,
}

#[derive(FsStore)]
pub struct Task {
    pub input: StoreFile,
    pub output: StoreFile,
}

#[derive(FsStore)]
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
        judger: impl judger::Judger,
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> Result<judger::TaskReport, RuntimeError> {
        let wd = judger.working_dir();
        let Subm { source } = subm;

        eprintln!("编译源文件");
        let term = compile_in_wd(source, &wd, "main")?;

        // Compile Error
        if !term.status.ok() {
            return Ok({
                let mut r = judger::TaskReport {
                    status: judger::Status::CompileError(term.status),
                    time: term.cpu_time,
                    memory: term.memory,
                    // todo: add log
                    payload: Vec::new(),
                };
                let _ = r.add_payload("compile log", wd.join("main.c.log"));
                r
            });
        }

        eprintln!("复制 IO 文件");
        copy_in_wd(&mut task.input, &wd, "input")?;
        copy_in_wd(&mut task.output, &wd, "answer")?;

        let s = SingletonBuilder::new(wd.join("main"))
            .stdin(wd.join("input"))
            .stdout(wd.join("output"))
            .stderr(wd.join("log"))
            .set_limits(|_| judger::sandbox::unix::Limitation {
                real_time: Some(meta.time_limit),
                cpu_time: Some((meta.time_limit, meta.time_limit)),
                virtual_memory: Some((meta.memory_limit, meta.memory_limit)),
                real_memory: Some(meta.memory_limit),
                stack_memory: Some((meta.memory_limit, meta.memory_limit)),
                output_memory: Some((meta.memory_limit, meta.memory_limit)),
                fileno: Some((5, 5)),
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

        report
            .payload
            .push(("checker log".into(), TruncStr::new(r.1, TRUNCATE_LEN)));
        if !r.0 {
            report.status = judger::Status::WrongAnswer;
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use judger::DefaultJudger;
    use store::Handle;

    use crate::{data::StoreFile, problem::JudgeProblem, Checker};

    use super::{Meta, Subm, Task, Traditional};

    impl StoreFile {
        /// use for testing
        pub(crate) fn create_tmp(content: impl AsRef<str>) -> Self {
            let mut file = tempfile::tempfile().unwrap();
            std::io::Write::write(&mut file, content.as_ref().as_bytes()).unwrap();
            Self {
                file,
                file_type: judger::FileType::Plain,
            }
        }
    }

    #[test]
    fn test_a_plus_b() {
        let a = Traditional;
        let dir = tempfile::tempdir().unwrap();
        let wd = Handle::new(dir);
        let jd = DefaultJudger::new(wd);
        let mut meta = Meta {
            checker: Checker::FileCmp,
            time_limit: 5000,
            memory_limit: 256 << 20,
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

        let report = a.judge_task(jd, &mut meta, &mut task, &mut subm).unwrap();
        if let judger::Status::Accepted = report.status {
        } else {
            panic!("not accepted")
        }
        dbg!(report);
    }
}
