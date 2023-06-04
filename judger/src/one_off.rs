//! Experimental. one off mode: 自定义评测，目前只支持 C++ 语言

use std::path::PathBuf;

use sandbox::{
    unix::{Limitation, SingletonBuilder},
    Builder, ExecSandBox,
};

use crate::{lang::Compile, Error, Status, TaskReport};

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff<L: Compile> {
    lang: L,
    source: PathBuf,
    stdin: Option<PathBuf>,
    /// 工作目录，默认值为 [`std::env::current_dir()`]
    working_dir: PathBuf,
    // time_limit: u64,
    // memory_limit: u64,
}

impl<L: Compile> OneOff<L> {
    /// 新建一个 OneOff，工作目录默认为 cwd（生成可执行文件的路径）
    pub fn new(source: PathBuf, stdin: Option<PathBuf>, lang: L) -> Self {
        Self {
            lang,
            source,
            stdin,
            working_dir: std::env::current_dir().unwrap(),
            // time_limit: 1000,
            // memory_limit: 256 * 1024 * 1024,
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

            let cpl = self.lang.compile(&self.source, &dest);
            let term = cpl.exec_sandbox()?;
            let st = term.status.clone();
            if st != sandbox::Status::Ok {
                let mut r: TaskReport = term.into();
                r.status = Status::CompileError(st);
                return Ok(r);
            }
            eprintln!("编译成功");
            let out = self.working_dir.join("main.output");
            let s = SingletonBuilder::new(dest)
                .set_limits(|_| Limitation {
                    real_time: Some(2000),
                    cpu_time: Some((2000, 2000)),
                    virtual_memory: Some((512 * 1024 * 1024, 512 * 1024 * 1024)),
                    real_memory: Some(512 * 1024 * 1024),
                    stack_memory: Some((512 * 1024 * 1024, 512 * 1024 * 1024)),
                    output_memory: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
                    fileno: Some((6, 6)),
                })
                .build()
                .unwrap();
            let term = s.exec_fork()?;
            let mut r: TaskReport = term.into();
            r.add_payload("stdout", out)?;
            Ok(r)
        } else {
            todo!()
        }
    }
}
