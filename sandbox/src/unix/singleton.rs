use crate::{
    error::{msg_err, UniError},
    unix::Limitation,
    vec_str_to_vec_cstr, MemoryLimitExceededKind, Status, Termination, TimeLimitExceededKind,
};
use core::panic;
use nix::{
    errno::Errno,
    libc,
    sys::{resource::getrusage, wait::waitpid},
    sys::{
        resource::{setrlimit, Resource, UsageWho},
        signal::{self, Signal},
        time::TimeValLike,
        wait::WaitStatus,
    },
    unistd::{execve, fork, setpgid, ForkResult, Pid},
};
use std::{ffi::CString, path::PathBuf, thread, time::Instant};

/// 执行单个可执行文件
#[derive(Debug)]
pub struct Singleton {
    limits: Limitation,
    exec_path: String,
    arguments: Vec<String>,
    envs: Vec<String>,
}

impl Singleton {
    fn exec_child(&self) -> Result<(), UniError> {
        setpgid(Pid::from_raw(0), Pid::from_raw(0))?;
        // 提前计算好需要的东西
        let (path, args, env) = (
            &CString::new(self.exec_path.clone())?,
            &vec_str_to_vec_cstr(&self.arguments)?,
            &vec_str_to_vec_cstr(&self.envs)?,
        );

        if let Some((s, h)) = self.limits.cpu_time {
            setrlimit(Resource::RLIMIT_CPU, s / 1000, h / 1000)?;
        }
        macro_rules! setlim {
            ($i:ident, $r:ident) => {
                if let Some((s, h)) = self.limits.$i {
                    setrlimit(Resource::$r, s, h)?;
                }
            };
        }
        setlim!(virtual_memory, RLIMIT_AS);
        setlim!(stack_memory, RLIMIT_STACK);
        setlim!(output_memory, RLIMIT_FSIZE);
        setlim!(fileno, RLIMIT_NOFILE);

        execve(path, args, env)?;
        Ok(())
    }
    fn exec_parent(&self, child: Pid, start: Instant) -> Result<Termination, UniError> {
        // let mut elapse = None;
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        let handle = self.limits.real_time.map(|tl| {
            let child_inhandle = child.clone();
            let st = start.clone();
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
                            if st.elapsed().as_millis() > tl as u128 {
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
        let waitres = waitpid(child, None)?;
        let duration = start.elapsed();
        let u = getrusage(UsageWho::RUSAGE_CHILDREN)?;
        let real_time = duration.as_millis() as u64;
        let cpu_time =
            (u.user_time().num_milliseconds() + u.system_time().num_milliseconds()) as u64;
        let memory = (u.max_rss() * 1024) as u64;

        macro_rules! real_tle {
            () => {
                self.limits.real_time.is_some() && self.limits.real_time.unwrap() < real_time
            };
        }
        macro_rules! real_mle {
            () => {
                self.limits.real_memory.is_some() && self.limits.real_memory.unwrap() < memory
            };
        }
        let status: Status = match waitres {
            WaitStatus::Exited(_, exit_code) => {
                println!("子进程正常退出");
                if real_mle!() {
                    Status::MemoryLimitExceeded(MemoryLimitExceededKind::Real)
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
                tx.send(())?; // 终止计时的线程
                println!("对计时线程发送终止信号");
                h.join().map_err(|e| format!("{:?}", e))?; // 等待线程结束
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
    fn exec_sandbox(&self) -> Result<crate::Termination, UniError> {
        let start = Instant::now();
        match unsafe { fork() } {
            Err(_) => msg_err("fork failed"),
            Ok(ForkResult::Parent { child }) => self.exec_parent(child, start),
            Ok(ForkResult::Child) => match self.exec_child() {
                Ok(_) => unsafe { libc::_exit(0) },
                Err(_) => unsafe { libc::_exit(1) },
            },
        }
    }
}

/// 在构建 Singleton 时的参数类型，主要用于 [`crate::sigton`].
pub struct ArgStr(pub String);

impl From<String> for ArgStr {
    fn from(value: String) -> Self {
        ArgStr(value)
    }
}
impl From<&str> for ArgStr {
    fn from(value: &str) -> Self {
        ArgStr(value.to_string())
    }
}
impl From<&PathBuf> for ArgStr {
    fn from(value: &PathBuf) -> Self {
        match value.to_str() {
            Some(s) => s.into(),
            None => panic!("invalid argument!"),
        }
    }
}
impl From<PathBuf> for ArgStr {
    fn from(value: PathBuf) -> Self {
        (&value).into()
    }
}

/// 创建一个 Singleton，请使用对应的 macro [`crate::sigton`].
pub struct SingletonBuilder {
    limits: Limitation,
    exec_path: Option<String>,
    arguments: Vec<String>,
    envs: Vec<String>,
}

macro_rules! lim_fn {
    ($i:ident) => {
        /// 添加资源限制（一个参数）
        pub fn $i(&mut self, val: u64) -> &mut Self {
            self.limits.$i = Some(val);
            self
        }
    };
    ($i:ident, 2) => {
        /// 添加资源限制（soft and hard）
        pub fn $i(&mut self, soft: u64, hard: u64) -> &mut Self {
            self.limits.$i = Some((soft, hard));
            self
        }
    };
}

impl SingletonBuilder {
    /// 新建一个 builder
    #[cold]
    pub fn new() -> Self {
        SingletonBuilder {
            limits: Limitation {
                real_time: None,
                cpu_time: None,
                virtual_memory: None,
                real_memory: None,
                stack_memory: None,
                output_memory: None,
                fileno: None,
            },
            exec_path: None,
            arguments: Vec::new(),
            envs: Vec::new(),
        }
    }
    /// 设置可执行文件的路径
    #[cold]
    pub fn exec_path<T: Into<ArgStr>>(&mut self, str: T) -> &mut Self {
        let ArgStr(s) = str.into();
        self.exec_path = Some(s);
        self
    }
    /// 在参数列表末尾添加一个参数
    #[inline]
    pub fn push_arg<T: Into<ArgStr>>(&mut self, arg: T) -> &mut Self {
        let ArgStr(s) = arg.into();
        self.arguments.push(s);
        self
    }
    /// 添加一个环境变量
    #[inline]
    pub fn add_env<T: Into<ArgStr>>(&mut self, val: T) -> &mut Self {
        let ArgStr(s) = val.into();
        self.envs.push(s);
        self
    }

    lim_fn!(real_time);
    lim_fn!(real_memory);
    lim_fn!(cpu_time, 2);
    lim_fn!(virtual_memory, 2);
    lim_fn!(stack_memory, 2);
    lim_fn!(output_memory, 2);
    lim_fn!(fileno, 2);
    // #[cold]
    // pub fn real_time(&mut self, val: u64) -> &mut Self {
    //     self.limits.real_time = Some(val);
    //     self
    // }
    /// 完成构建
    #[cold]
    pub fn finish(self) -> Singleton {
        Singleton {
            limits: self.limits,
            exec_path: match self.exec_path {
                Some(s) => s,
                None => panic!("singleton 缺少可执行文件的路径"),
            },
            arguments: self.arguments,
            envs: self.envs,
        }
    }
}

/// 使用宏规则来快速初始化 Singleton.
///
/// 目前支持的命令语法有：
///
/// - 指定可执行文件：`exec: {path}`;
/// - 指定完整的执行命令：`cmd: {args...}`;
/// - 设置环境变量：`env: {vars...};`
/// - 限制 CPU 执行时间、虚拟内存、栈空间、输出内存、文件指针数：
///   `lim cpu_time|virtual_memory|stack|output|fileno: {soft} {hard}`;
/// - 限制实际运行时间、实际使用内存：`lim real_time|real_memory: {time}`;
///
/// `exec`、`cmd` 和 `env` 可以接受任何实现了 [Into]<ArgStr> 的类型。
/// 按照官方文档，对于类型 T 你只需要对 ArgStr 实现 [From]<T> trait 就可以自动实现 [Into] trait。
///
/// 时间的单位是毫秒，内存的单位是字节。
///
/// Example:
///
/// ```rust
/// use sandbox::sigton;
/// let s = sigton! {
///     exec: "/usr/bin/sleep";
///     cmd: "sleep" "2";
///     env: "PATH=/usr/local/bin:/usr/bin" "A=b";
///     lim cpu_time: 1000 3000;
///     lim real_time: 2000;
///     lim real_memory: 256 * 1024 * 1024;
///     lim virtual_memory: 256 * 1024 * 1024 1024 * 1024 * 1024;
///     lim stack: 256 * 1024 * 1024 1024 * 1024 * 1024;
///     lim output: 256 * 1024 * 1024 1024 * 1024 * 1024;
///     lim fileno: 10 10;
/// };
/// ```
#[macro_export]
macro_rules! sigton {
    ($( $( $cmds:ident )+ $(: $( $args:expr )*)? );*$(;)?) => {
        // 使用新建代码块的方式解决定义域问题
        {
            let mut __singleton__ = $crate::unix::SingletonBuilder::new();
            $( sigton!("ln" __singleton__ $( $cmds )+ $(: $( $args ),* )? ) );*;
            __singleton__.finish()
        }
    };
    // 解析子命令，$self 表示定义的 __singleton__
    ("ln" $self:ident exec : $arg:expr) => {
        $self.exec_path($arg)
    };
    ("ln" $self:ident cmd : $( $args:expr ),+ ) => {
        // $self.arguments(vec![$( $arg ),*])
        $( $self.push_arg($args) );+
    };
    ("ln" $self:ident env : $( $args:expr ),+ ) => {
        $( $self.add_env($args) );+
    };
    ("ln" $self:ident lim cpu_time : $soft:expr,$hard:expr) => {
        $self.cpu_time($soft, $hard)
    };
    ("ln" $self:ident lim real_time : $val:expr) => {
        $self.real_time($val)
    };
    ("ln" $self:ident lim virtual_memory: $soft:expr,$hard:expr) => {
        $self.virtual_memory($soft, $hard)
    };
    ("ln" $self:ident lim stack: $soft:expr,$hard:expr) => {
        $self.stack_memory($soft, $hard)
    };
    ("ln" $self:ident lim real_memory : $val:expr) => {
        $self.real_memory($val)
    };
    ("ln" $self:ident lim output: $soft:expr,$hard:expr) => {
        $self.output_memory($soft, $hard)
    };
    ("ln" $self:ident lim fileno: $soft:expr,$hard:expr) => {
        $self.fileno($soft, $hard)
    };
}

#[cfg(test)]
mod tests {
    use super::Status;
    use crate::ExecSandBox;
    use crate::TimeLimitExceededKind;
    // use super::un

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_free() -> Result<(), super::UniError> {
        let ls_path = if cfg!(target_os = "linux") {
            "/usr/bin/ls"
        } else {
            "/bin/ls"
        };

        let singleton = sigton! {
            exec: ls_path;
            cmd: "ls" "." "-l";
        };
        let term = singleton.exec_fork()?;
        assert_eq!(term.status, Status::Ok);
        println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_tle_real() -> Result<(), super::UniError> {
        let sleep_path = if cfg!(target_os = "linux") {
            "/usr/bin/sleep"
        } else {
            "/bin/sleep"
        };
        // sleep 5 秒，触发 TLE
        // sleep 不会占用 CPU，因此触发 real time tle
        let singleton = sigton! {
            exec: sleep_path;
            cmd: "sleep" "2";
            lim cpu_time: 1000 3000;
            lim real_time: 1000;
        };
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
    fn singleton_env() -> Result<(), super::UniError> {
        let env_path = if cfg!(target_os = "linux") {
            "/usr/bin/env"
        } else {
            "/bin/env"
        };

        let singleton = sigton! {
            exec: env_path;
            cmd: "env";
            env: "DIR=/usr" "A=b"
        };

        let term = singleton.exec_fork()?;
        assert_eq!(term.status, Status::Ok);
        // println!("termination: {:?}", term);
        Ok(())
    }
}
