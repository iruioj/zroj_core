use crate::{
    check_limit,
    error::{self, msg_error, Error},
    vec_str_to_vec_cstr, Limitation, Status, Termination,
};
use nix::{
    libc,
    sys::{resource::getrusage, wait::waitpid},
    sys::{
        resource::{setrlimit, Resource, UsageWho},
        time::TimeValLike,
        wait::WaitStatus,
    },
    unistd::{execve, fork, ForkResult, Pid},
};
use core::panic;
use std::{ffi::CString, time::Instant};

/// 执行单个可执行文件
#[derive(Debug)]
pub struct Singleton {
    limits: Vec<Limitation>,
    exec_path: String,
    arguments: Vec<String>,
    envs: Vec<String>,
}

#[cfg(target_os = "linux")]
impl Singleton {
    fn exec_child(&self) -> Result<(), error::Error> {
        // todo: do some limitation
        for lim in self.limits.iter() {
            match lim {
                Limitation::CpuTime(s, h) => {
                    setrlimit(Resource::RLIMIT_CPU, (s / 1000) as u64, (h / 1000) as u64)?
                }
                Limitation::VirtualMemory(s, h) => {
                    setrlimit(Resource::RLIMIT_AS, *s as u64, *h as u64)?
                }
                Limitation::StackMemory(s, h) => {
                    setrlimit(Resource::RLIMIT_STACK, *s as u64, *h as u64)?
                }
                Limitation::OutputMemory(s, h) => {
                    setrlimit(Resource::RLIMIT_FSIZE, *s as u64, *h as u64)?
                }
                Limitation::Fileno(s, h) => {
                    setrlimit(Resource::RLIMIT_NOFILE, *s as u64, *h as u64)?
                }
                // Limitation::ActualMemory(_) => todo!(),
                // Limitation::RealTime(tl) => setrlimit(Resource::, soft_limit, hard_limit),
                _ => (),
            }
        }

        execve(
            &CString::new(self.exec_path.clone())?,
            &vec_str_to_vec_cstr(&self.arguments)?,
            &vec_str_to_vec_cstr(&self.envs)?,
        )?;
        Ok(())
    }
    fn exec_parent(&self, child: Pid, start: Instant) -> Result<crate::Termination, error::Error> {
        match waitpid(child, None)? {
            WaitStatus::Exited(_, exit_code) => {
                let duration = start.elapsed();
                let u = getrusage(UsageWho::RUSAGE_CHILDREN)?;
                let mut term = Termination {
                    status: Status::Ok,
                    real_time: duration.as_millis() as i64,
                    cpu_time: u.user_time().num_milliseconds() + u.system_time().num_milliseconds(),
                    memory: u.max_rss() * 1024,
                };
                if exit_code != 0 {
                    term.status = Status::RuntimeError(exit_code, None)
                } else {
                    for lim in &self.limits {
                        if let Some(status) = check_limit(&term, lim) {
                            term.status = status;
                            break;
                        }
                    }
                }
                Ok(term)
            }
            WaitStatus::Signaled(_, signal, _) => Ok(Termination::from(signal)),
            WaitStatus::Stopped(_, signal) => Ok(Termination::from(signal)),
            _ => msg_error("未知的等待结果".to_string()),
        }
    }
}

impl crate::ExecSandBox for Singleton {
    fn exec_sandbox(&self) -> Result<crate::Termination, Error> {
        let start = Instant::now();
        match unsafe { fork() } {
            Err(_) => msg_error("fork failed".to_string()),
            Ok(ForkResult::Parent { child }) => self.exec_parent(child, start),
            Ok(ForkResult::Child) => match self.exec_child() {
                Ok(_) => unsafe { libc::_exit(0) },
                Err(_) => unsafe { libc::_exit(1) },
            },
        }
    }
}

/// 创建一个 Singleton，请使用对应的 macro [`crate::sigton`].
pub struct SingletonBuilder {
    limits: Vec<Limitation>,
    exec_path: Option<String>,
    arguments: Option<Vec<String>>,
    envs: Option<Vec<String>>,
}

impl SingletonBuilder {
    /// 新建一个 builder
    #[cold]
    pub fn new() -> Self {
        SingletonBuilder {
            limits: vec![],
            exec_path: None,
            arguments: None,
            envs: None,
        }
    }
    /// 设置可执行文件的路径
    #[cold]
    pub fn exec_path<T: std::string::ToString>(&mut self, str: T) -> &mut Self {
        self.exec_path = Some(str.to_string());
        self
    }
    /// 设置可执行文件的参数，注意从第二个参数开始才是传递给进程的参数，
    /// 第一个参数代表可执行文件本身的运行时名字
    #[cold]
    pub fn arguments(&mut self, args: Vec<String>) -> &mut Self {
        self.arguments = Some(args);
        self
    }
    /// 设置环境变量
    #[cold]
    pub fn envs(&mut self, args: Vec<String>) -> &mut Self {
        self.envs = Some(args);
        self
    }
    /// 添加资源限制
    #[cold]
    pub fn add_limit(&mut self, lim: Limitation) -> &mut Self {
        self.limits.push(lim);
        self
    }
    /// 完成构建
    #[cold]
    pub fn finish(self) -> Singleton {
        Singleton {
            limits: self.limits,
            exec_path: match self.exec_path {
                Some(s) => s,
                None => panic!("singleton 缺少可执行文件的路径"),
            },
            arguments: match self.arguments {
                Some(args) => args,
                None => vec![],
            },
            envs: match self.envs {
                Some(envs) => envs,
                None => vec![],
            },
        }
    }
}

/// 使用宏规则来快速初始化 Singleton.
///
/// 目前支持的命令语法有：
///
/// - 指定可执行文件：`exec: {path}`;
/// - 指定完整的执行命令：`cmd: {args...}`;
/// - 限制 CPU 执行时间、虚拟内存、栈空间、输出内存、文件指针数：
///   `lim cpu_time|virtual_memory|stack|output|fileno: {soft} {hard}`;
/// - 限制实际运行时间、实际使用内存：`lim real_time|real_memory: {time}`;
///
/// 时间的单位是毫秒，内存的单位是字节。无效指令会被自动忽略。
///
/// Example:
///
/// ```rust
/// use sandbox::sigton;
/// let s = sigton! {
///     exec: "/usr/bin/sleep";
///     cmd: "sleep" "2";
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
        $self.arguments(vec![$( $args.to_string() ),*])
    };
    ("ln" $self:ident env : $( $args:expr ),+ ) => {
        $self.envs(vec![$( $args.to_string() ),*])
    };
    ("ln" $self:ident lim cpu_time : $soft:expr,$hard:expr) => {
        $self.add_limit($crate::Limitation::CpuTime($soft, $hard))
    };
    ("ln" $self:ident lim real_time : $val:expr) => {
        $self.add_limit($crate::Limitation::RealTime($val))
    };
    ("ln" $self:ident lim virtual_memory: $soft:expr,$hard:expr) => {
        $self.add_limit($crate::Limitation::VirtualMemory($soft, $hard))
    };
    ("ln" $self:ident lim stack: $soft:expr,$hard:expr) => {
        $self.add_limit($crate::Limitation::StackMemory($soft, $hard))
    };
    ("ln" $self:ident lim real_memory : $val:expr) => {
        $self.add_limit($crate::Limitation::RealMemory($val))
    };
    ("ln" $self:ident lim output: $soft:expr,$hard:expr) => {
        $self.add_limit($crate::Limitation::OutputMemory($soft, $hard))
    };
    ("ln" $self:ident lim fileno: $soft:expr,$hard:expr) => {
        $self.add_limit($crate::Limitation::Fileno($soft, $hard))
    };
}

#[cfg(test)]
mod tests {
    use super::{error,Status};
    use crate::ExecSandBox;
    use crate::TimeLimitExceededKind;
    // use super::un

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_free() -> Result<(), error::Error> {
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
    fn singleton_tle_real() -> Result<(), error::Error> {
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
    fn singleton_env() -> Result<(), error::Error> {
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
