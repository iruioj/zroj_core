//! Experimental. one off mode: 自定义评测，目前只支持 C++ 语言

use std::path::PathBuf;

use sandbox::ExecSandBox;

use crate::{Error, JudgeResult, LangOption, Status};

/// OneOff 只需要处理简单的时空限制即可
/// OneOff 假定你已经在 working_dir（默认当前目录）准备好了相关的原始文件
#[cfg(all(unix))]
pub struct OneOff<L: LangOption> {
    lang: L,
    source: PathBuf,
    /// 工作目录
    working_dir: PathBuf,
    // compile_args: Vec<String>,
    // stdin: Option<PathBuf>,
    // time_limit: u64,
    // memory_limit: u64,
}

impl<L: LangOption> OneOff<L> {
    pub fn new(source: PathBuf, lang: L) -> Self {
        return Self {
            lang,
            source,
            working_dir: std::env::current_dir().unwrap(),
            // compile_args: Vec::new(),
            // stdin: None,
            // time_limit: 1000,
            // memory_limit: 256 * 1024 * 1024,
        };
    }
    // pub fn set_args(&mut self, args: Vec<String>) -> &mut Self {
    //     self.compile_args = args;
    //     self
    // }
    pub fn set_wd(&mut self, dir: PathBuf) -> &mut Self {
        self.working_dir = dir;
        self
    }
    // pub fn set_stdin(&mut self, file: PathBuf) -> &mut Self {
    //     self.stdin = Some(file);
    //     self
    // }
    pub fn exec(&self) -> Result<JudgeResult, Error> {
        if cfg!(all(unix)) {
            // 可执行文件名
            let dest = self.working_dir.join("main");

            let cpl = self.lang.build_sigton(&self.source, &dest);
            let term = match cpl.exec_sandbox() {
                Ok(r) => r,
                Err(e) => {
                    return Ok(JudgeResult {
                        status: Status::CompileError,
                        msg: e.to_string().into(),
                        time: 0,
                        memory: 0,
                    })
                }
            };

            Ok(JudgeResult {
                status: Status::Accepted,
                msg: "compile succeed".into(),
                time: term.real_time,
                memory: term.memory,
            })
        } else {
            todo!()
        }
    }
}
