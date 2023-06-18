//! Experimental. one off mode: 自定义评测

use std::path::PathBuf;

use sandbox::{
    unix::{Limitation, SingletonBuilder},
    Builder, ExecSandBox, Elapse, Memory, time, mem,
};

use crate::{lang::Compile, Error, Status, TaskReport};

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可 TODO：自定义时空限制
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff<L: Compile> {
    lang: L,
    source: PathBuf,
    stdin: Option<PathBuf>,
    /// 工作目录，默认值为 [`std::env::current_dir()`]
    working_dir: PathBuf,
    time_limit: Elapse,
    memory_limit: Memory,
}

impl<L: Compile> OneOff<L> {
    /// 新建一个 OneOff 评测环境，工作目录默认为 cwd（生成可执行文件的路径），编译语言为 lang
    pub fn new(source: PathBuf, stdin: Option<PathBuf>, lang: L) -> Self {
        Self {
            lang,
            source,
            stdin,
            working_dir: std::env::current_dir().unwrap(),
            time_limit: time!(1s),
            memory_limit: mem!(512mb),
        }
    }
    pub fn set_wd(&mut self, dir: PathBuf) -> &mut Self {
        self.working_dir = dir;
        self
    }
    pub fn exec(&self) -> Result<TaskReport, Error> {
        eprintln!("source = {}", self.source.display());
        if cfg!(all(unix)) {
            // 可执行文件名
            let dest = self.working_dir.join("main");
            let clog = self.working_dir.join("compile.log");
            // compilation
            let cpl = self.lang.compile_sandbox(&self.source, &dest, &clog);
            let term = cpl.exec_sandbox()?;
            let st = term.status.clone();
            if st != sandbox::Status::Ok {
                let mut r: TaskReport = term.into();
                r.status = Status::CompileError(st);
                return Ok(r);
            }
            eprintln!("编译成功");
            // execution
            let out = self.working_dir.join("main.output");
            let log = self.working_dir.join("main.log");
            assert!(dest.exists());
            let mut s = SingletonBuilder::new(dest)
                .set_limits(|_| Limitation {
                    real_time: self.time_limit.into(),
                    cpu_time: self.time_limit.into(),
                    virtual_memory: self.memory_limit.into(),
                    real_memory: self.memory_limit.into(),
                    stack_memory: self.memory_limit.into(),
                    output_memory: mem!(64mb).into(),
                    fileno: 6.into(),
                })
                .stdout(&out)
                .stderr(&log);
            if let Some(stdin) = &self.stdin {
                s = s.stdin(stdin);
            }
            let s = s.build().unwrap();
            let term = s.exec_fork()?;
            dbg!(&term);
            let mut r: TaskReport = term.into();
            // ignore error
            let _ = r.add_payload("compile log", clog);
            let _ = r.add_payload("stdout", out);
            let _ = r.add_payload("stderr", log);
            Ok(r)
        } else {
            todo!()
        }
    }
}
