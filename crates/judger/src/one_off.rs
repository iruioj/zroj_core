//! Experimental. one off mode: 自定义评测

use crate::{Judger, SourceFile, StoreFile, TaskMeta, TaskReport};
use anyhow::Context;
use sandbox::{
    unix::{Lim, Limitation, SingletonConfig},
    Elapse, Memory,
};
use store::Handle;

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可 TODO：自定义时空限制
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(unix)]
pub struct OneOff {
    file: SourceFile,
    stdin: StoreFile,
    /// 工作目录，默认值为 [`std::env::current_dir()`]
    working_dir: Handle,
    time_limit: Elapse,
    memory_limit: Memory,
    output_limit: Memory,
    fileno_limit: u64,
}

impl OneOff {
    /// 新建一个 OneOff 评测环境，工作目录默认为 cwd（生成可执行文件的路径），编译语言为 lang
    pub fn new(file: SourceFile, stdin: StoreFile) -> Self {
        Self {
            file,
            stdin,
            working_dir: Handle::new(std::env::current_dir().unwrap()),
            time_limit: Elapse::from_sec(1),
            memory_limit: Memory::from_mb(1024),
            output_limit: Memory::from_mb(128),
            fileno_limit: 10,
        }
    }
    pub fn set_wd(&mut self, dir: Handle) -> &mut Self {
        self.working_dir = dir;
        self
    }
    pub fn exec(&mut self) -> anyhow::Result<TaskReport> {
        let judger = crate::DefaultJudger::<&str>::new(self.working_dir.clone(), None);

        judger
            .working_dir()
            .prepare_empty_dir()
            .context("init working dir")?;

        eprintln!("编译源文件");
        let crate::Compilation {
            termination: term,
            log_payload,
            execfile,
        } = judger.compile(&mut self.file, "main-pre")?;

        // Compile Error
        if !term.status.ok() {
            return Ok(crate::TaskReport {
                meta: crate::TaskMeta {
                    score_rate: 0.0,
                    status: crate::Status::CompileError(Some(term.status)),
                    time: term.cpu_time,
                    memory: term.memory,
                },
                payload: vec![("compile log".into(), log_payload)],
            });
        }

        let mut execfile = execfile.context("compile succeed but execfile not found")?;
        let exec = judger.copy_file(&mut execfile, "main")?;

        let input = judger.copy_store_file(&mut self.stdin, "input")?;
        let output = judger.clear_dest("output")?;
        let log = judger.clear_dest("log")?;

        let s = SingletonConfig::new(exec.to_string())
            .push_args(["main"])
            .stdin(input.to_string())
            .stdout(output.to_string())
            .stderr(log.to_string())
            .set_limits(|_| Limitation {
                real_time: Lim::Double(self.time_limit, Elapse::from(self.time_limit.ms() * 2)),
                cpu_time: self.time_limit.into(),
                virtual_memory: self.memory_limit.into(),
                real_memory: self.memory_limit.into(),
                stack_memory: self.memory_limit.into(),
                output_memory: self.output_limit.into(),
                fileno: self.fileno_limit.into(),
            });

        let term = judger.exec_sandbox(s)?;

        let status: crate::Status = term.status.into();
        let mut report = TaskReport::new(TaskMeta {
            score_rate: status.direct_score_rate(),
            status,
            time: term.cpu_time,
            memory: term.memory,
        })
        .try_add_payload("stdout", output)
        .try_add_payload("stderr", log);
        report.payload.push(("compile log".into(), log_payload));

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use crate::{report, FileType};

    use super::*;

    #[test]
    fn test_it() {
        let source = SourceFile::from_str(
            r#"
#include<iostream>
using namespace std;
int main() {
    // write for(;;); directly will result in SIGTRAP on MacOS
    for(;;) cout << "h" << endl;
}
"#,
            FileType::GnuCpp17O2,
        );
        let input = StoreFile::from_str(r"1 2", FileType::Plain);
        let mut oneoff = OneOff::new(source, input);
        let dir = tempfile::TempDir::new().unwrap();
        oneoff.set_wd(Handle::new(dir.path()));
        let rep = oneoff.exec().unwrap();
        assert_eq!(rep.meta.status, report::Status::TimeLimitExceeded);
        drop(dir);
    }
}
