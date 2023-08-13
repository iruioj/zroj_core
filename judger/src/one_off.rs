//! Experimental. one off mode: 自定义评测

use std::path::PathBuf;

use sandbox::{mem, time, unix::Limitation, Elapse, Memory};
use store::Handle;

use crate::{lang::Compile, Error, Status, StoreFile, TaskReport};

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可 TODO：自定义时空限制
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff {
    file: StoreFile,
    stdin: StoreFile,
    /// 工作目录，默认值为 [`std::env::current_dir()`]
    working_dir: Handle,
    time_limit: Elapse,
    memory_limit: Memory,
    output_limit: Memory,
    fileno_limit: u64,
    sandbox_exec: Option<PathBuf>,
}

impl OneOff {
    /// 新建一个 OneOff 评测环境，工作目录默认为 cwd（生成可执行文件的路径），编译语言为 lang
    pub fn new(file: StoreFile, stdin: StoreFile, sandbox_exec: Option<PathBuf>) -> Self {
        Self {
            file,
            stdin,
            working_dir: Handle::new(std::env::current_dir().unwrap()),
            time_limit: time!(1s),
            memory_limit: mem!(512mb),
            output_limit: mem!(64mb),
            fileno_limit: 6,
            sandbox_exec,
        }
    }
    pub fn set_wd(&mut self, dir: Handle) -> &mut Self {
        self.working_dir = dir;
        self
    }
    #[cfg(all(unix))]
    pub fn exec(&mut self) -> Result<TaskReport, Error> {
        use crate::TaskMeta;
        use sandbox::{
            unix::{Lim, SingletonBuilder},
            Builder, ExecSandBox,
        };
        use std::process::Command;

        let src = self
            .working_dir
            .join(String::from("main") + self.file.file_type.ext());
        self.file.copy_to(&src).expect("cannot copy source file");
        // 可执行文件名
        let dest = self.working_dir.join("main");
        let clog = self.working_dir.join("compile.log");
        // compilation
        eprintln!("编译...");
        if !self.file.file_type.compileable() {
            let r = TaskReport {
                meta: TaskMeta {
                    score: 0.0,
                    status: Status::CompileError(None),
                    time: 0.into(),
                    memory: 0.into(),
                },
                payload: Vec::new(),
            };
            return Ok(r);
        }
        let term = self
            .file
            .file_type
            .compile_sandbox(&src, &dest, &clog)
            .exec_fork()?;
        let st = term.status.clone();
        if st != sandbox::Status::Ok {
            let mut r = TaskReport {
                meta: TaskMeta {
                    score: 0.0,
                    status: Status::CompileError(Some(st)),
                    time: term.cpu_time,
                    memory: term.memory,
                },
                payload: Vec::new(),
            };
            let _ = r.add_payload("compile log", clog);
            eprintln!("编译失败");
            dbg!(&self.working_dir);
            return Ok(r);
        }
        drop(term);
        eprintln!("编译成功");
        // execution
        let out = self.working_dir.join("main.out");
        let log = self.working_dir.join("main.log");
        let input = self.working_dir.join("main.in");
        self.stdin.copy_to(&input).expect("cannot copy input file");
        assert!(dest.as_ref().exists());
        let term: sandbox::Termination = if let Some(sandbox_exec) = &self.sandbox_exec {
            eprintln!("sandbox executable path: {}", sandbox_exec.display());
            let term_f = self.working_dir.join("term.json");
            let mut p = Command::new(sandbox_exec)
                .arg(dest.path())
                .arg("--stdin")
                .arg(input.path())
                .arg("--stdout")
                .arg(out.path())
                .arg("--stderr")
                .arg(log.path())
                .arg("--lim")
                .arg(
                    Limitation {
                        real_time: Lim::Double(
                            self.time_limit,
                            Elapse::from(self.time_limit.ms() * 2),
                        ),
                        cpu_time: self.time_limit.into(),
                        virtual_memory: self.memory_limit.into(),
                        real_memory: self.memory_limit.into(),
                        stack_memory: self.memory_limit.into(),
                        output_memory: self.output_limit.into(),
                        fileno: self.fileno_limit.into(),
                    }
                    .to_string(),
                )
                .arg("--save")
                .arg(term_f.path())
                .spawn()
                .unwrap();
            let status = p.wait().unwrap();
            assert!(status.success());
            let term_file = term_f.open_file().unwrap();
            serde_json::from_reader(&term_file).unwrap()
        } else {
            let s = SingletonBuilder::new(dest)
                .set_limits(|_| Limitation {
                    real_time: Lim::Double(self.time_limit, Elapse::from(self.time_limit.ms() * 2)),
                    cpu_time: self.time_limit.into(),
                    virtual_memory: self.memory_limit.into(),
                    real_memory: self.memory_limit.into(),
                    stack_memory: self.memory_limit.into(),
                    output_memory: self.output_limit.into(),
                    fileno: self.fileno_limit.into(),
                })
                .stdout(&out)
                .stderr(&log)
                .stdin(input)
                .build()
                .unwrap();
            eprintln!("开始运行选手程序");
            let term = s.exec_fork()?;
            eprintln!("程序运行结束");
            term
        };

        let mut r: TaskReport = TaskReport {
            meta: TaskMeta {
                score: 0.0,
                status: term.status.into(),
                time: term.cpu_time,
                memory: term.memory,
            },
            payload: Vec::new(),
        };
        r.meta.score = r.meta.status.score_rate();
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
