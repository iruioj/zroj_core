use sandbox::unix::{Lim, Limitation, Singleton};
use sandbox::{Elapse, ExecSandBox, Memory};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::ffi::CString;
use std::path::PathBuf;
use store::Handle;

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

pub const COMPILE_LIM: Limitation = Limitation {
    real_time: Lim::Double(Elapse::from_sec(10), Elapse::from_sec(20)),
    cpu_time: Lim::Single(Elapse::from_sec(10)),
    virtual_memory: Lim::Single(Memory::from_mb(4096)),
    real_memory: Lim::Single(Memory::from_mb(4096)),
    stack_memory: Lim::Single(Memory::from_mb(4096)),
    output_memory: Lim::Single(Memory::from_mb(1024)),
    fileno: Lim::Single(200),
};

impl GnuCpp {
    fn compile_sandbox(
        &self,
        source: &Handle,
        dest: &Handle,
        log: &Handle,
    ) -> Box<dyn ExecSandBox> {
        let r = Singleton::new(&self.gpp_path)
            .push_args([CString::new("g++").unwrap()])
            .push_args(
                self.extra_args
                    .iter()
                    .map(|s| CString::new(s.as_bytes()))
                    .collect::<Result<Vec<CString>, _>>()
                    .unwrap(),
            )
            .push_args([
                source.to_cstring(),
                CString::new("-o").unwrap(),
                dest.to_cstring(),
            ])
            .with_current_env()
            .set_limits(|_| COMPILE_LIM)
            .stderr(log.to_cstring());
        Box::new(r)
    }
}

/// 内置的支持的文件类型
#[derive(Serialize, Deserialize, Clone, Debug, TsType, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
            FileType::GnuCpp20O2 | FileType::GnuCpp17O2 | FileType::GnuCpp14O2 => "cpp",
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

impl FileType {
    /// 生成一个编译指令，将源文件编译为可执行文件
    ///
    /// - source: 源文件路径
    /// - dest: 编译产生的可执行文件的路径
    /// - log: 编译日志文件
    pub fn compile_sandbox(
        &self,
        source: &Handle,
        dest: &Handle,
        log: &Handle,
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
            FileType::Rust => {
                let r = Singleton::new(&crate::which("rustc").unwrap())
                    .push_args([
                        CString::new("rustc").unwrap(),
                        source.to_cstring(),
                        CString::new("-o").unwrap(),
                        dest.to_cstring(),
                    ])
                    .with_current_env()
                    .set_limits(|_| COMPILE_LIM);
                Box::new(r)
            }
            _ => unimplemented!(),
        }
    }
}
