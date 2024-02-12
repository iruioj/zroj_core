use sandbox::unix::{Lim, Limitation, Singleton};
use sandbox::{Elapse, ExecSandBox, Memory};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

/// 一个 Compile 是指对 **单个源文件** 指定的语言，并提供对应的编译指令
pub trait Compile {
    /// 生成一个编译指令
    ///
    /// - source: 源文件路径
    /// - dest: 编译产生的可执行文件的路径
    /// - log: 编译日志文件
    fn compile_sandbox(
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
        // dbg!(&gpp_path);
        let extra_args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
        GnuCpp {
            gpp_path,
            extra_args,
        }
    }
}

impl Compile for GnuCpp {
    fn compile_sandbox(
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
            Singleton::new(CString::new(self.gpp_path.as_os_str().as_bytes()).unwrap())
                .push_arg([CString::new("g++").unwrap()])
                .push_arg(
                    self.extra_args
                        .iter()
                        .map(|s| CString::new(s.as_bytes()))
                        .collect::<Result<Vec<CString>, _>>()
                        .unwrap(),
                )
                .push_arg([
                    CString::new(source.as_ref().as_os_str().as_bytes()).unwrap(),
                    CString::new("-o").unwrap(),
                    CString::new(dest.as_ref().as_os_str().as_bytes()).unwrap(),
                ])
                .push_env(
                    envs.iter()
                        .map(|s| CString::new(s.as_bytes()))
                        .collect::<Result<Vec<CString>, _>>()
                        .unwrap(),
                )
                .set_limits(|_| Limitation {
                    real_time: Lim::Double(Elapse::from_sec(10), Elapse::from_sec(20)),
                    cpu_time: Elapse::from_sec(10).into(),
                    virtual_memory: Memory::from_mb(4096).into(),
                    real_memory: Memory::from_mb(4096).into(),
                    stack_memory: Memory::from_mb(4096).into(),
                    output_memory: Memory::from_mb(1024).into(),
                    fileno: 200.into(),
                })
                .stderr(CString::new(log.as_ref().as_os_str().as_bytes()).unwrap()),
        )
    }
}

/// 内置的支持的文件类型
#[derive(Serialize, Deserialize, Clone, Debug, TsType, PartialEq, Eq, Hash)]
pub enum FileType {
    #[serde(rename = "gnu_cpp20_o2")]
    GnuCpp20O2,
    #[serde(rename = "gnu_cpp17_o2")]
    GnuCpp17O2,
    #[serde(rename = "gnu_cpp14_o2")]
    GnuCpp14O2,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "python3")]
    Python,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "gnu_assembly")]
    Assembly,
}

impl FileType {
    /// 获取文件类型对应的后缀名
    pub fn ext(&self) -> &'static str {
        match &self {
            FileType::GnuCpp20O2 => "cpp",
            FileType::GnuCpp17O2 => "cpp",
            FileType::GnuCpp14O2 => "cpp",
            FileType::Plain => "txt",
            FileType::Python => "py",
            FileType::Rust => "rs",
            FileType::Assembly => "s",
        }
    }
    pub fn compileable(&self) -> bool {
        !matches!(self, FileType::Plain)
    }
}

impl Compile for FileType {
    fn compile_sandbox(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
        log: impl AsRef<Path>,
    ) -> Box<dyn ExecSandBox> {
        match self {
            FileType::GnuCpp20O2 => {
                GnuCpp::new(None, vec!["-std=c++2a", "-O2", "-Wall", "-Wextra"])
                    .compile_sandbox(source, dest, log)
            }
            FileType::GnuCpp17O2 => {
                GnuCpp::new(None, vec!["-std=c++17", "-O2", "-Wall", "-Wextra"])
                    .compile_sandbox(source, dest, log)
            }
            FileType::GnuCpp14O2 => {
                GnuCpp::new(None, vec!["-std=c++14", "-O2", "-Wall", "-Wextra"])
                    .compile_sandbox(source, dest, log)
            }
            FileType::Plain => panic!("a plain file should never be compiled"),
            _ => todo!(),
        }
    }
}
