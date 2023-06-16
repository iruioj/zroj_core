use sandbox::unix::{Limitation, SingletonBuilder};
use sandbox::{Builder, ExecSandBox};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

use crate::sha_hash::{ShaHash, Update};

/// 一个 Compile 是指对 **单个源文件** 指定的语言，并提供对应的编译指令
pub trait Compile: ShaHash {
    /// 生成一个编译指令
    ///
    /// - source: 源文件路径
    /// - dest: 编译产生的可执行文件的路径
    /// - log: 编译日志文件
    #[cfg(all(unix))]
    fn compile(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
        log: impl AsRef<Path>,
    ) -> Box<dyn ExecSandBox>;
}

/// 使用 g++ 编译 C++ 源文件
pub struct GnuCpp {
    gpp_path: PathBuf,
    extra_args: Vec<String>,
}

impl GnuCpp {
    /// 默认编译器为 g++
    pub fn new(gpp_path: Option<PathBuf>, args: Vec<&'static str>) -> Self {
        let gpp_path = gpp_path.unwrap_or(crate::env::which("g++").unwrap());
        let extra_args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
        GnuCpp {
            gpp_path,
            extra_args,
        }
    }
}

impl ShaHash for GnuCpp {
    fn sha_hash(&self, state: &mut sha2::Sha256) {
        state.update(self.gpp_path.to_str().unwrap().as_bytes());
        for ele in &self.extra_args {
            state.update("$".as_bytes());
            state.update(ele.as_bytes());
        }
    }
}

impl Compile for GnuCpp {
    fn compile(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
        log: impl AsRef<Path>,
    ) -> Box<dyn ExecSandBox> {
        let mut envs = Vec::new();
        for (key, value) in std::env::vars() {
            envs.push(format!("{}={}", key, value));
        }
        Box::new(
            SingletonBuilder::new(&self.gpp_path)
                .push_arg("g++")
                .push_arg(&self.extra_args)
                .push_arg(source.as_ref())
                .push_arg("-o")
                .push_arg(dest.as_ref())
                .push_env(envs)
                .set_limits(|_| Limitation {
                    real_time: Some(10000), // 编译时间限制为 10s
                    cpu_time: Some((10000, 10000)),
                    virtual_memory: Some((1024 * 1024 * 1024, 1024 * 1024 * 1024)),
                    real_memory: Some(1024 * 1024 * 1024),
                    stack_memory: Some((1024 * 1024 * 1024, 1024 * 1024 * 1024)),
                    output_memory: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
                    fileno: Some((50, 50)),
                })
                .stderr(log)
                .build()
                .unwrap(),
        )
    }
}

/// 内置的支持的文件类型
#[derive(Serialize, Deserialize, Clone)]
pub enum FileType {
    #[serde(rename = "gnu_cpp20_o2")]
    GnuCpp20O2,
    #[serde(rename = "gnu_cpp17_o2")]
    GnuCpp17O2,
    #[serde(rename = "gnu_cpp14_o2")]
    GnuCpp14O2,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "python3")]
    Python,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "gnu_assembly")]
    Assembly,
}

/// 纯文本语言的 “编译器”：直接复制
struct PlainCompile {
    src: PathBuf,
    dest: PathBuf,
}

impl ExecSandBox for PlainCompile {
    fn exec_sandbox(&self) -> Result<sandbox::Termination, sandbox::SandboxError> {
        let mut dest = std::fs::File::create(&self.dest)?;
        let mut src = std::fs::File::open(&self.src)?;
        std::io::copy(&mut src, &mut dest)?;

        Ok(sandbox::Termination {
            status: sandbox::Status::Ok,
            real_time: 0,
            cpu_time: 0,
            memory: 0,
        })
    }
}
impl FileType {
    /// 获取文件类型对应的后缀名
    pub fn ext(&self) -> &'static str {
        match &self {
            FileType::GnuCpp20O2 => ".cpp",
            FileType::GnuCpp17O2 => ".cpp",
            FileType::GnuCpp14O2 => ".cpp",
            FileType::Plain => ".txt",
            FileType::Binary => ".blob",
            FileType::Python => ".py",
            FileType::Rust => ".rs",
            FileType::Assembly => ".s",
        }
    }
}

impl ShaHash for FileType {
    fn sha_hash(&self, state: &mut sha2::Sha256) {
        state.update(serde_json::to_string(self).unwrap().as_bytes())
    }
}

impl Compile for FileType {
    fn compile(&self, source: impl AsRef<Path>, dest: impl AsRef<Path>, log: impl AsRef<Path>) -> Box<dyn ExecSandBox> {
        match self {
            FileType::GnuCpp20O2 => {
                GnuCpp::new(None, vec!["-std=c++2a", "-O2"]).compile(source, dest, log)
            }
            FileType::GnuCpp17O2 => {
                GnuCpp::new(None, vec!["-std=c++17", "-O2"]).compile(source, dest, log)
            }
            FileType::GnuCpp14O2 => {
                GnuCpp::new(None, vec!["-std=c++14", "-O2"]).compile(source, dest, log)
            }
            FileType::Plain => Box::new(PlainCompile {
                src: source.as_ref().to_path_buf(),
                dest: dest.as_ref().to_path_buf(),
            }),
            _ => todo!(),
        }
    }
}
