//! Experimental. one off mode: 自定义评测

use sandbox::{
    mem, time,
    unix::{Limitation, SingletonBuilder},
    Builder, Elapse, ExecSandBox, Memory,
};
use store::Handle;

use crate::{lang::Compile, Error, Status, StoreFile, TaskReport};

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可 TODO：自定义时空限制
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff {
    file: StoreFile,
    stdin: Option<StoreFile>,
    /// 工作目录，默认值为 [`std::env::current_dir()`]
    working_dir: Handle,
    time_limit: Elapse,
    memory_limit: Memory,
    output_limit: Memory,
    fileno_limit: u64,
}

impl OneOff {
    /// 新建一个 OneOff 评测环境，工作目录默认为 cwd（生成可执行文件的路径），编译语言为 lang
    pub fn new(file: StoreFile, stdin: Option<StoreFile>) -> Self {
        Self {
            file,
            stdin,
            working_dir: Handle::new(std::env::current_dir().unwrap()),
            time_limit: time!(1s),
            memory_limit: mem!(512mb),
            output_limit: mem!(64mb),
            fileno_limit: 6,
        }
    }
    pub fn set_wd(&mut self, dir: Handle) -> &mut Self {
        self.working_dir = dir;
        self
    }
    #[cfg(all(unix))]
    pub fn exec(&mut self) -> Result<TaskReport, Error> {
        let src = self
            .working_dir
            .join(String::from("main") + self.file.file_type.ext());
        self.file.copy_to(&src).expect("cannot copy source file");
        // 可执行文件名
        let dest = self.working_dir.join("main");
        let clog = self.working_dir.join("compile.log");
        // compilation
        eprintln!("编译...");
        let term = self
            .file
            .file_type
            .compile_sandbox(&src, &dest, &clog)
            .exec_fork()?;
        let st = term.status.clone();
        if st != sandbox::Status::Ok {
            let mut r: TaskReport = term.into();
            let _ = r.add_payload("compile log", clog);
            r.status = Status::CompileError(st);
            eprintln!("编译失败");
            dbg!(&self.working_dir);
            return Ok(r);
        }
        eprintln!("编译成功");
        // execution
        let out = self.working_dir.join("main.out");
        let log = self.working_dir.join("main.log");
        assert!(dest.as_ref().exists());
        let mut s = SingletonBuilder::new(dest)
            .set_limits(|_| Limitation {
                real_time: self.time_limit.into(),
                cpu_time: self.time_limit.into(),
                virtual_memory: self.memory_limit.into(),
                real_memory: self.memory_limit.into(),
                stack_memory: self.memory_limit.into(),
                output_memory: self.output_limit.into(),
                fileno: self.fileno_limit.into(),
            })
            .stdout(&out)
            .stderr(&log);
        if let Some(stdin) = &mut self.stdin {
            let input = self.working_dir.join("main.in");
            stdin.copy_to(&input).expect("cannot copy input file");
            s = s.stdin(input);
        }
        let s = s.build().unwrap();
        let term = s.exec_fork()?;
        let mut r: TaskReport = term.into();
        // ignore error
        let _ = r.add_payload("compile log", clog).map_err(|e| dbg!(e));
        let _ = r.add_payload("stdout", out).map_err(|e| dbg!(e));
        let _ = r.add_payload("stderr", log).map_err(|e| dbg!(e));
        Ok(r)
    }
    #[cfg(not(unix))]
    pub fn exec(&mut self) -> Result<TaskReport, Error> {
        todo!()
    }
}
