use std::path::PathBuf;

/// 一个 LangOption 是指对 **单个源文件** 指定的语言，并提供对应的编译指令
pub trait LangOption {
    /// 生成一个编译指令
    /// 
    /// - source: 源文件路径
    /// - dest: 编译产生的可执行文件的路径
    #[cfg(all(unix))]
    fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton;
}

/// 使用 g++ 编译 C++ 源文件
pub struct GnuCpp {
    gpp_path: PathBuf,
    extra_args: Vec<String>,
}

impl GnuCpp {
    pub fn new(args: Vec<&'static str>) -> Self {
        let gpp_path = crate::env::which("g++").unwrap();
        let extra_args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
        GnuCpp {
            gpp_path,
            extra_args,
        }
    }
}

impl LangOption for GnuCpp {
    fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton {
        let mut envs = Vec::new();
        for (key, value) in std::env::vars() {
            envs.push(format!("{}={}", key, value));
        }
        let envs = envs;

        sandbox::sigton! {
            exec: &self.gpp_path;
            cmd: "g++" self.extra_args.clone() source "-o" dest;
            env: envs;
            lim cpu_time: 10000 10000; // 10s
            lim real_time: 10000;
            lim real_memory: 1024 * 1024 * 1024;
            lim virtual_memory: 1024 * 1024 * 1024 1024 * 1024 * 1024;
            lim stack: 1024 * 1024 * 1024 1024 * 1024 * 1024;
            lim output: 64 * 1024 * 1024 64 * 1024 * 1024;
            lim fileno: 50 50;
        }
    }
}
pub fn gnu_cpp20_o2() -> GnuCpp {
    GnuCpp::new(vec!["-std=c++2a", "-O2"])
}
pub fn gnu_cpp17_o2() -> GnuCpp {
    GnuCpp::new(vec!["-std=c++17", "-O2"])
}
pub fn gnu_cpp14_o2() -> GnuCpp {
    GnuCpp::new(vec!["-std=c++14", "-O2"])
}
/// 内置的支持的语言
pub enum Builtin {
    GnuCpp20O2,
    GnuCpp17O2,
    GnuCpp14O2,
}
impl LangOption for Builtin {
    fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton {
        match self {
            Builtin::GnuCpp20O2 => gnu_cpp20_o2().build_sigton(source, dest),
            Builtin::GnuCpp17O2 => gnu_cpp17_o2().build_sigton(source, dest),
            Builtin::GnuCpp14O2 => gnu_cpp14_o2().build_sigton(source, dest),
        }
    }
}
