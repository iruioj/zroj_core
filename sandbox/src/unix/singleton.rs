use crate::{
    error::{ChildError, SandboxError},
    unix::Limitation,
    vec_str_to_vec_cstr, Builder, Elapse, Memory, MemoryLimitExceededKind, Status, Termination,
    TimeLimitExceededKind,
};
use nix::{
    errno::Errno,
    fcntl::{self, OFlag},
    libc,
    sys::{resource::getrusage, stat::Mode, wait::waitpid},
    sys::{
        resource::{setrlimit, Resource, UsageWho},
        signal::{self, Signal},
        time::TimeValLike,
        wait::WaitStatus,
    },
    unistd,
};
use std::{
    ffi::CString,
    os::{fd::RawFd, unix::prelude::OsStrExt},
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

/// 执行单个可执行文件
#[derive(Debug)]
pub struct Singleton {
    limits: Limitation,
    exec_path: PathBuf,
    arguments: Vec<String>,
    envs: Vec<String>,
    /// 为 None 表示不提供/不获取 读入/输出
    stdin: Option<PathBuf>,
    stdout: Option<PathBuf>,
    stderr: Option<PathBuf>,
}

// for async-signal-safty
fn c_write(s: &[u8]) {
    unsafe {
        libc::write(
            libc::STDERR_FILENO,
            s.as_ptr() as *const std::ffi::c_void,
            s.len(),
        );
    }
}
fn open_file(path: impl AsRef<std::path::Path>, flag: OFlag) -> Result<RawFd, ChildError> {
    fcntl::open(
        path.as_ref().as_os_str(),
        flag,
        // mode is for creating new file
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .map_err(|errno| {
        ChildError::OpenFile(errno.to_string(), path.as_ref().to_path_buf(), flag.bits())
    })
}
fn dup(to: RawFd, from: RawFd) -> Result<RawFd, ChildError> {
    nix::unistd::dup2(to, from).map_err(|errno| ChildError::Dup(errno.to_string(), to, from))
}
impl Singleton {
    fn exec_child(&self) -> Result<(), ChildError> {
        c_write(b"start exec child\n");
        unistd::setpgid(unistd::Pid::from_raw(0), unistd::Pid::from_raw(0))
            .map_err(|errno| ChildError::SetPGID(errno.to_string()))?;
        c_write(b"pgid set\n");
        // 提前计算好需要的东西
        let path = CString::new(self.exec_path.as_os_str().as_bytes())
            .expect("exec_path contains null char");
        let args = vec_str_to_vec_cstr(&self.arguments).expect("arguments contains null char");
        let env = vec_str_to_vec_cstr(&self.envs).expect("enviroment arguments contains null char");
        // redirect standard IO
        if let Some(stdin) = &self.stdin {
            let fd = open_file(stdin, OFlag::O_RDONLY)?;
            dup(fd, libc::STDIN_FILENO)?;
            c_write(b"stdin redirected\n");
        }
        if let Some(stdout) = &self.stdout {
            let fd = open_file(stdout, OFlag::O_WRONLY | OFlag::O_TRUNC | OFlag::O_CREAT)?;
            dup(fd, libc::STDOUT_FILENO)?;
            c_write(b"stdout redirected\n");
        }
        if let Some(stderr) = &self.stderr {
            let fd = open_file(stderr, OFlag::O_WRONLY | OFlag::O_TRUNC | OFlag::O_CREAT)?;
            dup(fd, libc::STDERR_FILENO)?;
        } else {
            c_write(b"stderr not redirected\n");
        }
        // set resource limit
        macro_rules! setlim {
            ($i:ident, $r:ident, $f:ident) => {
                match self.limits.$i {
                    crate::unix::Lim::None => {}
                    crate::unix::Lim::Single(s) => {
                        setrlimit(Resource::$r, s.$f(), s.$f()).map_err(|errno| {
                            ChildError::SetRlimit(
                                errno.to_string(),
                                stringify!($r).into(),
                                s.$f(),
                                s.$f(),
                            )
                        })?;
                    }
                    crate::unix::Lim::Double(s, h) => {
                        setrlimit(Resource::$r, s.$f(), h.$f()).map_err(|errno| {
                            ChildError::SetRlimit(
                                errno.to_string(),
                                stringify!($r).into(),
                                s.$f(),
                                s.$f(),
                            )
                        })?;
                    }
                }
            };
        }
        setlim!(cpu_time, RLIMIT_CPU, sec);
        // macos 对于内存的控制有自己的见解，如果在这里限制的话会 RE
        // 这意味着 macos 上的安全性会低一些
        #[cfg(not(target_os = "macos"))]
        setlim!(virtual_memory, RLIMIT_AS, byte);
        #[cfg(not(target_os = "macos"))]
        setlim!(stack_memory, RLIMIT_STACK, byte);
        setlim!(output_memory, RLIMIT_FSIZE, byte);
        setlim!(fileno, RLIMIT_NOFILE, into);
        if self.stderr.is_none() {
            c_write(b"resource limited\n");
        }
        // todo: set syscall limit
        unistd::execve(&path, &args, &env).map_err(|errno| {
            ChildError::Execve(
                errno.to_string(),
                self.exec_path.clone(),
                self.arguments.clone(),
                self.envs.clone(),
            )
        })?;
        unreachable!("execve returns infallible")
    }
    fn exec_parent(&self, child: unistd::Pid, start: Instant) -> Result<Termination, SandboxError> {
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        // 如果有实际运行时间限制，就开一个计时线程
        let handle = self.limits.real_time.to_soft_option().map(|tl| {
            let child_inhandle = child;
            let st = start;
            thread::spawn(move || {
                loop {
                    thread::sleep(std::time::Duration::from_millis(500));
                    println!("beep...");
                    match rx.try_recv() {
                        Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                            println!("[计时线程] 子进程先结束");
                            break;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            if tl < st.elapsed().into() {
                                println!("[计时线程] 子进程超时");
                                break;
                            }
                        }
                    }
                }
                match signal::killpg(child_inhandle, Signal::SIGKILL) {
                    Ok(_) => {
                        println!("[计时线程] 成功杀死子进程组");
                    }
                    Err(err) => {
                        if err == Errno::ESRCH {
                            println!("[计时线程] 杀死子进程：子进程已经结束");
                        } else {
                            println!("[计时线程] 杀死子进程组出错（忽略）：{}", err);
                        }
                    }
                };
            })
        });
        let waitres = waitpid(child, None).expect("wait child process error");
        let duration = start.elapsed();
        let u = getrusage(UsageWho::RUSAGE_CHILDREN).expect("getrusage error");
        let real_time: Elapse = duration.into();
        let cpu_time = Elapse::from(
            (u.user_time().num_milliseconds() + u.system_time().num_milliseconds()) as u64,
        );
        // on macos, the maximum resident set size is measured in bytes (see man getrusage)
        #[cfg(target_os = "macos")]
        let memory = Memory::from((u.max_rss()) as u64);
        #[cfg(not(target_os = "macos"))]
        let memory = Memory::from((u.max_rss() * 1024) as u64);

        macro_rules! real_tle {
            () => {
                !self.limits.real_time.check(&real_time)
            };
        }
        macro_rules! real_mle {
            () => {
                !self.limits.real_memory.check(&memory)
            };
        }
        let status: Status = match waitres {
            WaitStatus::Exited(_, exit_code) => {
                println!("子进程正常退出");
                if real_mle!() {
                    Status::MemoryLimitExceeded(MemoryLimitExceededKind::Real(memory))
                } else if real_tle!() {
                    Status::TimeLimitExceeded(TimeLimitExceededKind::Real)
                } else if exit_code != 0 {
                    Status::RuntimeError(exit_code, None)
                } else {
                    Status::Ok
                }
            }
            WaitStatus::Signaled(_, signal, _) => {
                println!("子进程被信号终止");
                if signal == signal::SIGKILL && real_tle!() {
                    println!("子进程被计时线程终止");
                    Status::TimeLimitExceeded(TimeLimitExceededKind::Real)
                } else {
                    Status::RuntimeError(0, Some(signal.to_string()))
                }
            }
            WaitStatus::Stopped(_, signal) => {
                println!("未知子进程状态");
                Status::RuntimeError(0, Some(signal.to_string()))
            }
            _ => panic!("未知状态"),
        };
        if let Some(h) = handle {
            if !h.is_finished() {
                let _ = tx.send(()).map_err(|e| {
                    println!("对计时线程发送终止信号出错：{e}, the corresponding receiver has already been deallocated");
                }); // 终止计时的线程
                println!("对计时线程发送终止信号");
                let _ = h
                    .join()
                    .map_err(|e| format!("等待计时线程结束出错：{:?}", e)); // 等待线程结束
                println!("计时线程正常结束");
            }
        }
        println!("主进程正常结束");
        Ok(Termination {
            status,
            real_time,
            cpu_time,
            memory,
        })
    }
}

impl crate::ExecSandBox for Singleton {
    fn exec_sandbox(&self) -> Result<crate::Termination, SandboxError> {
        let start = Instant::now();
        match unsafe { unistd::fork() } {
            Err(e) => Err(SandboxError::Fork(e.to_string())),
            Ok(unistd::ForkResult::Parent { child }) => self.exec_parent(child, start),
            Ok(unistd::ForkResult::Child) => match self.exec_child() {
                Ok(_) => unsafe { libc::_exit(0) },
                Err(_) => unsafe { libc::_exit(1) },
            },
        }
    }
}

/// 在构建 Singleton 时的参数类型，主要用于 [`crate::sigton`].
pub enum Arg {
    /// 单个参数
    Str(String),
    /// 多个参数
    Vec(Vec<String>),
    /// 缺省值
    Nothing,
}

impl From<String> for Arg {
    fn from(value: String) -> Self {
        Arg::Str(value)
    }
}
impl From<&str> for Arg {
    fn from(value: &str) -> Self {
        Arg::Str(value.to_string())
    }
}
impl From<&PathBuf> for Arg {
    fn from(value: &PathBuf) -> Self {
        match value.to_str() {
            Some(s) => s.into(),
            None => panic!("invalid argument!"),
        }
    }
}
impl From<PathBuf> for Arg {
    fn from(value: PathBuf) -> Self {
        (&value).into()
    }
}
impl From<&Path> for Arg {
    fn from(value: &Path) -> Self {
        match value.to_str() {
            Some(s) => s.into(),
            None => panic!("invalid argument!"),
        }
    }
}
impl From<&Vec<String>> for Arg {
    fn from(value: &Vec<String>) -> Self {
        value.to_owned().into()
    }
}
impl From<Vec<String>> for Arg {
    fn from(value: Vec<String>) -> Self {
        Arg::Vec(value)
    }
}

macro_rules! impl_option {
    ($typename:ty) => {
        impl From<Option<$typename>> for Arg {
            fn from(value: Option<$typename>) -> Self {
                if let Some(value) = value {
                    value.into()
                } else {
                    Arg::Nothing
                }
            }
        }
    };
}

impl_option!(String);
impl_option!(&str);
impl_option!(PathBuf);
impl_option!(&PathBuf);

/// 创建一个 Singleton，请使用对应的 macro [`crate::sigton`].
pub struct SingletonBuilder {
    limits: Limitation,
    exec_path: Option<PathBuf>,
    arguments: Vec<String>,
    envs: Vec<String>,
    stdin: Option<PathBuf>,
    stdout: Option<PathBuf>,
    stderr: Option<PathBuf>,
}

impl SingletonBuilder {
    #[deprecated]
    #[doc(hidden)]
    pub fn new_legacy() -> Self {
        SingletonBuilder {
            limits: Limitation::default(),
            stdin: None,
            stdout: None,
            stderr: None,
            exec_path: None,
            arguments: Vec::new(),
            envs: Vec::new(),
        }
    }
    #[deprecated]
    #[doc(hidden)]
    pub fn exec_path_legacy<T: Into<Arg>>(&mut self, str: T) -> &mut Self {
        match str.into() as Arg {
            Arg::Str(s) => self.exec_path = Some(s.into()),
            Arg::Vec(_) => panic!("invalid exec_path"),
            Arg::Nothing => {}
        };
        self
    }
    #[deprecated]
    #[doc(hidden)]
    pub fn push_arg_legacy<T: Into<Arg>>(&mut self, arg: T) -> &mut Self {
        match arg.into() as Arg {
            Arg::Str(s) => self.arguments.push(s),
            Arg::Vec(mut v) => self.arguments.append(&mut v),
            Arg::Nothing => {}
        };
        self
    }
    #[deprecated]
    #[doc(hidden)]
    pub fn add_env<T: Into<Arg>>(&mut self, val: T) -> &mut Self {
        match val.into() as Arg {
            Arg::Str(s) => self.envs.push(s),
            Arg::Vec(mut v) => self.envs.append(&mut v),
            Arg::Nothing => {}
        };
        self
    }
    #[deprecated]
    #[doc(hidden)]
    pub fn set_stdin(&mut self, val: impl Into<Arg>) -> &mut Self {
        self.stdin = match val.into() as Arg {
            Arg::Str(s) => Some(s.into()),
            Arg::Vec(_) => panic!("invalid args"),
            Arg::Nothing => None,
        };
        self
    }
    #[deprecated]
    #[doc(hidden)]
    pub fn set_stdout(&mut self, val: impl Into<Arg>) -> &mut Self {
        self.stdout = match val.into() as Arg {
            Arg::Str(s) => Some(s.into()),
            Arg::Vec(_) => panic!("invalid args"),
            Arg::Nothing => None,
        };
        self
    }
}

impl Builder for SingletonBuilder {
    type Target = Singleton;

    fn build(self) -> Result<Self::Target, SandboxError> {
        dbg!(&self.exec_path);
        dbg!(&self.arguments);
        Ok(Singleton {
            limits: self.limits,
            exec_path: self.exec_path.unwrap(),
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: self.stderr,
            arguments: self.arguments,
            envs: self.envs,
        })
    }
}

// new API
impl SingletonBuilder {
    /// Create a new builder with the path of executable
    pub fn new(exec: impl AsRef<Path>) -> Self {
        SingletonBuilder {
            limits: Limitation::default(),
            stdin: None,
            stdout: None,
            stderr: None,
            exec_path: Some(exec.as_ref().to_path_buf()),
            arguments: Vec::new(),
            envs: Vec::new(),
        }
    }
    /// set the path of input file, which will be rediected to stdin.
    pub fn stdin(mut self, arg: impl AsRef<Path>) -> Self {
        self.stdin = Some(arg.as_ref().to_path_buf());
        self
    }
    /// set the path of output file, which will be rediected to stdout.
    pub fn stdout(mut self, arg: impl AsRef<Path>) -> Self {
        self.stdout = Some(arg.as_ref().to_path_buf());
        self
    }
    /// set the path of error output file, which will be rediected to stderr.
    pub fn stderr(mut self, arg: impl AsRef<Path>) -> Self {
        self.stderr = Some(arg.as_ref().to_path_buf());
        self
    }
    /// add an argument to the end of argument list
    pub fn push_arg(mut self, arg: impl Into<Arg>) -> Self {
        match arg.into() as Arg {
            Arg::Str(s) => {
                self.arguments.push(s);
            }
            Arg::Vec(mut v) => {
                self.arguments.append(&mut v);
            }
            Arg::Nothing => {}
        }
        self
    }
    /// add an argument to the end of environment list
    pub fn push_env(mut self, arg: impl Into<Arg>) -> Self {
        match arg.into() as Arg {
            Arg::Str(s) => {
                self.envs.push(s);
            }
            Arg::Vec(mut v) => {
                self.envs.append(&mut v);
            }
            Arg::Nothing => {}
        }
        self
    }
    /// set resource limitation
    pub fn set_limits(mut self, modifier: impl FnOnce(Limitation) -> Limitation) -> Self {
        self.limits = modifier(self.limits);
        self
    }
}

#[cfg(test)]
mod tests {
    use singleton::SingletonBuilder;

    use super::Status;
    use crate::unix::singleton;
    use crate::unix::Lim;
    use crate::Builder;
    use crate::ExecSandBox;
    use crate::TimeLimitExceededKind;
    // use super::un

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_free() -> Result<(), super::SandboxError> {
        let ls_path = if cfg!(target_os = "linux") {
            "/usr/bin/ls"
        } else {
            "/bin/ls"
        };

        let singleton = SingletonBuilder::new(ls_path)
            .push_arg("ls")
            .push_arg("-l")
            .push_arg(".")
            .build()?;

        let term = singleton.exec_fork()?;
        assert_eq!(term.status, Status::Ok);
        println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_tle_real() -> Result<(), super::SandboxError> {
        let sleep_path = if cfg!(target_os = "linux") {
            "/usr/bin/sleep"
        } else {
            "/bin/sleep"
        };
        // sleep 5 秒，触发 TLE
        // sleep 不会占用 CPU，因此触发 real time tle
        let singleton = SingletonBuilder::new(sleep_path)
            .push_arg("sleep")
            .push_arg("2")
            .set_limits(|mut l| {
                l.cpu_time = Lim::Double(1000.into(), 3000.into());
                l.real_time = Lim::Single(1000.into());
                l
            })
            .build()?;

        let term = singleton.exec_fork()?;
        assert_eq!(
            term.status,
            Status::TimeLimitExceeded(TimeLimitExceededKind::Real)
        );
        // println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_env() -> Result<(), super::SandboxError> {
        let env_path = "/usr/bin/env";

        let singleton = SingletonBuilder::new(env_path)
            .push_arg("env")
            .push_env("DIR=/usr")
            .push_env("A=b")
            .build()?;

        let term = singleton.exec_fork()?;
        assert_eq!(term.status, Status::Ok);
        // println!("termination: {:?}", term);
        Ok(())
    }
}
