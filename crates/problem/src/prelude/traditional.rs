use std::ffi::CString;

use crate::{
    data::StoreFile,
    judger_framework::{JudgeTask, LogMessage},
    Checker, Override,
};
use anyhow::Context;
use judger::{
    sandbox::{
        unix::{Lim, Singleton},
        Elapse, ExecSandBox, Memory,
    },
    truncstr::{TruncStr, TRUNCATE_LEN},
    SourceFile,
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
    /// 输出限制
    #[meta]
    pub output_limit: Memory,
}

impl Override<Meta> for &'_ () {
    fn over(self, _: &mut Meta) {}
}

#[derive(FsStore, Debug)]
pub struct Task {
    pub input: StoreFile,
    pub output: StoreFile,
}

/// Traditional problems submission contains only a single source file,
/// which reads from stdin and outputs to stdout.
#[derive(FsStore, Debug)]
pub struct Subm {
    pub source: SourceFile,
}

/// 传统题评测
pub struct Traditional;

impl JudgeTask for Traditional {
    type T = Task;
    type M = Meta;
    type Subm = Subm;

    // 先写了一个粗糙的，后面再来错误处理
    fn judge_task(
        judger: &mut impl judger::Judger<LogMessage>,
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> anyhow::Result<judger::TaskReport> {
        judger
            .working_dir()
            .prepare_empty_dir()
            .context("init working dir")?;

        let Subm { source } = subm;

        let judger::Compilation {
            termination: term,
            log_payload,
            execfile,
        }: judger::Compilation = judger.cachable_block(
            |judger, source| {
                eprintln!("编译源文件");
                judger.compile(source, "main-pre")
            },
            source,
        )?;

        // Compile Error
        if !term.status.ok() {
            return Ok(judger::TaskReport {
                meta: judger::TaskMeta {
                    score_rate: 0.0,
                    status: judger::Status::CompileError(Some(term.status)),
                    time: term.cpu_time,
                    memory: term.memory,
                },
                // todo: add log
                payload: vec![("compile log".into(), log_payload)],
            });
        }

        let mut execfile = execfile.context("compile succeed but execfile not found")?;
        dbg!(execfile.metadata().unwrap().permissions());
        let exec = judger.copy_file(&mut execfile, "main")?;

        let input = judger.copy_store_file(&mut task.input, "input")?;
        let answer = judger.copy_store_file(&mut task.output, "answer")?;

        let output = judger.clear_dest("output")?;
        let log = judger.clear_dest("log")?;

        let s = Singleton::new(exec.path())
            .push_args([CString::new("main").unwrap()])
            .stdin(input.to_cstring())
            .stdout(output.to_cstring())
            .stderr(log.to_cstring())
            .set_limits(|_| judger::sandbox::unix::Limitation {
                real_time: Lim::Double(meta.time_limit, meta.time_limit * 1.1),
                cpu_time: meta.time_limit.into(),
                virtual_memory: meta.memory_limit.into(),
                real_memory: meta.memory_limit.into(),
                stack_memory: meta.memory_limit.into(),
                output_memory: meta.output_limit.into(),
                fileno: 5.into(),
            });

        let term = s.exec_sandbox().unwrap();
        let term_status = term.status.clone();

        let mut report = judger::TaskReport {
            meta: judger::TaskMeta {
                score_rate: 0.0,
                status: term.status.into(),
                time: term.cpu_time,
                memory: term.memory,
            },
            payload: Vec::new(),
        };
        report.meta.score_rate = report.meta.status.direct_score_rate();
        report.payload.push(("compile log".into(), log_payload));
        let _ = report.add_payload("stdin", &input);
        let _ = report.add_payload("stdout", &output);
        let _ = report.add_payload("answer", &answer);
        let _ = report.add_payload("stderr", &log);

        if !term_status.ok() {
            return Ok(report);
        }

        // check answer
        let r = meta.checker.check(judger, &input, &output, &answer);

        report.meta.score_rate = r.as_ref().map(|o| o.0).unwrap_or(0.);
        report.payload.push((
            "checker log".into(),
            TruncStr::new(
                match r {
                    Ok(s) => s.1,
                    Err(s) => format!("{s:#?}"),
                },
                TRUNCATE_LEN,
            ),
        ));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::{Meta, Subm, Task, Traditional};
    use crate::{data::StoreFile, judger_framework::JudgeTask, Checker};
    use judger::{
        sandbox::{Elapse, Memory},
        DefaultJudger, SourceFile,
    };
    use store::Handle;

    #[test]
    fn test_a_plus_b() {
        let dir = tempfile::tempdir().unwrap();
        let wd = Handle::new(dir);
        let mut jd = DefaultJudger::new(wd, None);
        let mut meta = Meta {
            checker: Checker::FileCmp,
            time_limit: Elapse::from_sec(5),
            memory_limit: Memory::from_mb(256),
            output_limit: Memory::from_mb(64),
        };
        let mut task = Task {
            input: StoreFile::from_str("1 2", judger::FileType::Plain),
            output: StoreFile::from_str("3\n", judger::FileType::Plain),
        };
        let mut subm = Subm {
            source: SourceFile::from_str(
                r#"#include<iostream>
                        using namespace std;
                        int main() {
                            int a, b;
                            cin >> a >> b;
                            cout << a + b << endl;
                        }
                        "#,
                judger::FileType::GnuCpp14O2,
            ),
        };

        let report = Traditional::judge_task(&mut jd, &mut meta, &mut task, &mut subm).unwrap();
        let judger::Status::Good = report.meta.status else {
            panic!("not accepted")
        };
        dbg!(report);
    }
}
