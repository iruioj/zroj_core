//! Experimental. one off mode: 自定义评测，目前只支持 C++ 语言

use std::path::PathBuf;

use sandbox::{sigton, ExecSandBox};

use crate::{lang::LangOption, Error, JudgeResult, Status};

/// OneOff 用于执行自定义测试，流程包含：编译、运行可执行文件。
///
/// OneOff 只需要处理简单的时空限制即可
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff<L: LangOption> {
    lang: L,
    source: PathBuf,
    stdin: Option<PathBuf>,
    /// 工作目录
    working_dir: PathBuf,
    // compile_args: Vec<String>,
    // time_limit: u64,
    // memory_limit: u64,
}

impl<L: LangOption> OneOff<L> {
    /// 新建一个 OneOff，工作目录默认为 cwd（生成可执行文件的路径）
    pub fn new(source: PathBuf, stdin: Option<PathBuf>, lang: L) -> Self {
        return Self {
            lang,
            source,
            stdin,
            working_dir: std::env::current_dir().unwrap(),
            // time_limit: 1000,
            // memory_limit: 256 * 1024 * 1024,
        };
    }
    pub fn set_wd(&mut self, dir: PathBuf) -> &mut Self {
        self.working_dir = dir;
        self
    }
    // pub fn set_stdin(&mut self, file: PathBuf) -> &mut Self {
    //     self.stdin = Some(file);
    //     self
    // }
    pub fn exec(&self) -> Result<JudgeResult, Error> {
        eprintln!("source = {}", self.source.display());
        if cfg!(all(unix)) {
            // 可执行文件名
            let dest = self.working_dir.join("main");

            let cpl = self.lang.build_sigton(&self.source, &dest);
            let term = match cpl.exec_sandbox() {
                Ok(r) => r,
                Err(e) => {
                    return Ok(JudgeResult {
                        status: Status::CompileFailed(e.clone()),
                        msg: e.to_string().into(),
                        time: 0,
                        memory: 0,
                        stdout: "".into(),
                        stderr: "".into(),
                    })
                }
            };
            if term.status != sandbox::Status::Ok {
                return Ok(JudgeResult {
                    status: Status::CompileError(term.status.clone()),
                    msg: format!("{:?}", term.status).into(),
                    time: term.real_time,
                    memory: term.memory,
                    stdout: "".into(),
                    stderr: "".into(),
                });
            }
            eprintln!("编译成功");
            let out = self.working_dir.join("main.output");
            let s = sigton! {
                exec: dest;
                stdin: self.stdin.clone();
                stdout: out.clone();
                lim cpu_time: 2000 2000; // 2s
                lim real_time: 2000;
                lim real_memory: 512 * 1024 * 1024;
                lim virtual_memory: 512 * 1024 * 1024 512 * 1024 * 1024;
                lim stack: 512 * 1024 * 1024 612 * 1024 * 1024;
                lim output: 64 * 1024 * 1024 64 * 1024 * 1024;
                lim fileno: 6 6;
            };
            let term = s.exec_fork()?;

            let judge_status = match term.status {
                sandbox::Status::Ok => Status::Accepted,
                sandbox::Status::RuntimeError(_, _) => Status::RuntimeError,
                sandbox::Status::MemoryLimitExceeded(_) => Status::MemoryLimitExceeded,
                sandbox::Status::TimeLimitExceeded(_) => Status::TimeLimitExceeded,
                sandbox::Status::OutputLimitExceeded => Status::OutputLimitExceeded,
                sandbox::Status::DangerousSyscall => Status::DangerousSyscall,
            };

            Ok(JudgeResult {
                status: judge_status,
                msg: "".into(),
                time: term.real_time,
                memory: term.memory,
                stdout: std::fs::read_to_string(out)
                    .map_err(|e| Error::IOError(e))?
                    .into(),
                stderr: "".into(),
            })
        } else {
            todo!()
        }
    }
}
