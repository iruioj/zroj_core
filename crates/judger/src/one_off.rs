//! Experimental. one off mode: 自定义评测

use sandbox::{unix::Limitation, Elapse, Memory};
use store::Handle;

use crate::{SourceFile, Status, StoreFile, TaskReport};

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
    #[cfg(unix)]
    pub fn exec(&mut self) -> anyhow::Result<TaskReport> {
        use crate::TaskMeta;
        use sandbox::{
            unix::{Lim, SingletonConfig},
            ExecSandBox,
        };

        let src = self
            .working_dir
            .join("main")
            .with_extension(self.file.file_type.ext());
        self.file.copy_to(&src).expect("cannot copy source file");
        // 可执行文件名
        let dest = self.working_dir.join("main");
        let clog = self.working_dir.join("compile.log");

        // FIXME
        // #[cfg(target_os = "macos")]
        // {
        //     let mut p = Command::new("g++")
        //         .arg(src.path())
        //         .arg("-o")
        //         .arg(dest.path())
        //         .spawn()
        //         .unwrap();
        //     let r = p.wait().unwrap();
        //     assert!(dest.path().is_file());
        //     assert!(r.success());
        // }

        // compilation
        eprintln!("编译...");
        if !self.file.file_type.compileable() {
            let r = TaskReport::new(TaskMeta::error_status(Status::CompileError(None)));
            return Ok(r);
        }
        let term = self
            .file
            .file_type
            .compile_sandbox(&src, &dest, &clog)
            .exec_sandbox()
            .unwrap();
        let st = term.status.clone();
        if st != sandbox::Status::Ok {
            let r = TaskReport::new(TaskMeta {
                score_rate: 0.0,
                status: Status::CompileError(Some(st)),
                time: term.cpu_time,
                memory: term.memory,
            })
            .try_add_payload("compile log", clog);
            return Ok(r);
        }
        eprintln!("编译成功 {:?}", dest);

        // execution
        let out = self.working_dir.join("main.out");
        let log = self.working_dir.join("main.log");
        let input = self.working_dir.join("main.in");
        self.stdin.copy_to(&input).expect("cannot copy input file");
        assert!(dest.as_ref().exists());
        let term: sandbox::Termination = {
            let s = SingletonConfig::new(dest.to_string())
                .set_limits(|_| Limitation {
                    real_time: Lim::Double(self.time_limit, Elapse::from(self.time_limit.ms() * 2)),
                    cpu_time: self.time_limit.into(),
                    virtual_memory: self.memory_limit.into(),
                    real_memory: self.memory_limit.into(),
                    stack_memory: self.memory_limit.into(),
                    output_memory: self.output_limit.into(),
                    fileno: self.fileno_limit.into(),
                })
                .stdout(out.to_string())
                .stderr(log.to_string())
                .stdin(input.to_string())
                .build();
            eprintln!("开始运行选手程序");
            // 为了避免 getrusage 数值累加，使用 exec_fork
            let term = s.exec_sandbox().unwrap();
            eprintln!("程序运行结束");
            term
        };
        let status: crate::Status = term.status.into();

        Ok(TaskReport::new(TaskMeta {
            score_rate: status.direct_score_rate(),
            status,
            time: term.cpu_time,
            memory: term.memory,
        })
        .try_add_payload("compile log", clog)
        .try_add_payload("stdout", out)
        .try_add_payload("stderr", log))
    }
    #[cfg(not(unix))]
    pub fn exec(&mut self) -> Result<TaskReport, Error> {
        todo!()
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
        // let mut oneoff = OneOff::new(source, input, Some("/Users/sshwy/zroj_core/target/debug/zroj-sandbox".into()));
        let mut oneoff = OneOff::new(source, input);
        let dir = tempfile::TempDir::new().unwrap();
        oneoff.set_wd(Handle::new(dir.path()));
        let rep = oneoff.exec().unwrap();
        assert_eq!(rep.meta.status, report::Status::TimeLimitExceeded);
        drop(dir);
    }
}
