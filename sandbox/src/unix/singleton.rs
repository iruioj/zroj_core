use crate::{
    unix::{share_mem, signal_safe, Limitation},
    Elapse, Memory, Status, Termination,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{CStr, CString},
    io::Write,
    time::Instant,
};

/// 执行单个可执行文件
#[derive(Debug, Serialize, Deserialize)]
pub struct Singleton {
    limits: Limitation,
    exec_path: CString,
    arguments: Vec<CString>,
    envs: Vec<CString>,
    /// 为 None 表示不提供/不获取 读入/输出
    stdin: Option<CString>,
    stdout: Option<CString>,
    stderr: Option<CString>,
}

impl Singleton {
    /// In child process, before calling execve, you should only execute
    /// async-signal-safe funtions, thus don't use unistd.
    fn exec_child(
        &self,
        path: &CStr,
        args: &[*mut std::ffi::c_char],
        env: &[*mut std::ffi::c_char],
        guard: signal_safe::SigblockGuard,
        shared: share_mem::GlobalShared,
    ) -> Result<(), signal_safe::Errno> {
        // register a handler for SIGCHLD to make sigsuspend work
        signal_safe::signal_echo(signal_safe::SIGCHLD);

        // at this time, all signals are blocked, so we can fork directly
        let pid_child = signal_safe::fork()?;

        // fork another child process to execute program
        if pid_child == 0 {
            signal_safe::set_self_grp();
            // redirect standard IO
            if let Some(stdin) = &self.stdin {
                let fd = signal_safe::open_read(stdin)?;
                signal_safe::dup2(fd, signal_safe::STDIN_FILENO);
            }
            if let Some(stdout) = &self.stdout {
                let fd = signal_safe::open_write(stdout)?;
                signal_safe::dup2(fd, signal_safe::STDOUT_FILENO);
            }
            if let Some(stderr) = &self.stderr {
                let fd = signal_safe::open_write(stderr)?;
                signal_safe::dup2(fd, signal_safe::STDERR_FILENO);
            } else {
                signal_safe::print_cstr(b"(child-child) stderr not redirected\n\x00");
            }

            // set resource limit
            macro_rules! setlim {
                ($i:ident, $r:ident, $f:ident) => {
                    match self.limits.$i {
                        crate::unix::Lim::None => {}
                        crate::unix::Lim::Single(s) => {
                            signal_safe::setrlimit(signal_safe::$r as i32, s.$f(), s.$f())?;
                        }
                        crate::unix::Lim::Double(s, h) => {
                            signal_safe::setrlimit(signal_safe::$r as i32, s.$f(), h.$f())?;
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
                signal_safe::print_cstr(b"(child-child) resource limited\n\x00");
            }
            drop(guard); // unblock signals
                         // todo: set syscall limit
            signal_safe::execve(path, args, env);
        }

        signal_safe::print_cstr(b"(child) fork a timmer\n\x00");

        // at this time, all signals are blocked, so we can fork directly
        let pid_timer = match self.limits.real_time {
            crate::unix::Lim::None => None,
            crate::unix::Lim::Single(s) | crate::unix::Lim::Double(s, _) => {
                let pid = signal_safe::fork()?;
                if pid == 0 {
                    // fork a process to setup timer
                    signal_safe::sleep(s.sec().clamp(0, u32::MAX as u64) as u32);
                    signal_safe::print_cstr(b"(child-timer) timmer exit\n\x00");
                    signal_safe::exit(0);
                }
                Some(pid)
            }
        };

        // can be interrupted either by timer or child process
        let mut timer_first = false;
        let mut child_status = None;

        signal_safe::print_cstr(b"(child) wait for tested process and timer\n\x00");

        let ru = 'outer: loop {
            // notice that sigsuspend only interrupts for signals whose action is
            // either calling handler function or exit (thus sometimes you need to
            // register handler for a signal to make it work).
            guard.suspend();
            signal_safe::print_cstr(b"(child) wait beep\n\x00");
            loop {
                // since all signals are blocked, SIGCHLD will not interrupt
                let r = signal_safe::waitpid(-1, signal_safe::WNOHANG);

                match r {
                    Ok((pid, status)) => {
                        if pid_timer.is_some_and(|pid_timer| pid_timer == pid) {
                            // normally the timer should be killed by signal. thus if timer returned first -> TLE
                            if status.exited() {
                                timer_first = true;
                                // at this time, child hasn't been reaped, thus it's pid is not freed
                                if let Err(e) = signal_safe::kill(pid_child, signal_safe::SIGKILL) {
                                    if self.stderr.is_none() {
                                        signal_safe::print_cstr(
                                            b"(child) timer kill child failed\n\x00",
                                        );
                                    }
                                    break 'outer Err(e);
                                }
                            } // otherwise timer is killed, ignored
                        } else if pid_child == pid {
                            child_status = Some(status);
                            if !timer_first {
                                // child return first
                                // at this time, timer hasn't been reaped, thus it's pid is not freed
                                if let Some(pid_timer) = pid_timer {
                                    if let Err(e) =
                                        signal_safe::kill(pid_timer, signal_safe::SIGKILL)
                                    {
                                        if self.stderr.is_none() {
                                            signal_safe::print_cstr(
                                                b"(child) kill timer failed\n\x00",
                                            );
                                        }
                                        break 'outer Err(e);
                                    }
                                }
                            }
                        } else if pid == 0 {
                            // if WNOHANG is specified and there are no stopped or exited children, 0 is returned
                            break;
                        } else {
                            // reap it and do nothing
                        }
                    }
                    Err(errno) => {
                        if errno.is_errno(signal_safe::ECHILD) {
                            // no child to wait
                            break 'outer share_mem::get_rusage();
                        } else {
                            if self.stderr.is_none() {
                                signal_safe::print_cstr(b"(child) child wait child failed\n\x00");
                            }
                            break 'outer Err(errno);
                        }
                    }
                }
            }
        }?;

        if !shared.try_set(share_mem::global_shared_t {
            rusage: ru,
            timer_first: if timer_first { 1 } else { 0 },
            status: child_status.map(|a| a.0).unwrap_or(-1),
        }) {
            signal_safe::print_cstr(b"(child) set shared memory error\n\x00");
            signal_safe::exit(1);
        }
        Ok(())
    }
    fn exec_parent(
        &self,
        child: i32,
        start: Instant,
        guard: signal_safe::SigblockGuard,
        shared: &share_mem::GlobalShared,
    ) -> anyhow::Result<Termination> {
        // do something before waiting for child process
        drop(guard);

        let (_, child_status) =
            signal_safe::waitpid(child, 0).context("parent wait child error")?;

        let real_time = start.elapsed().into();

        if !(child_status.exited() && child_status.exitstatus() == 0) {
            return Err(anyhow::anyhow!(
                "child (parent) not normally terminated, with status {}",
                child_status.0
            ));
        }
        let share_mem::global_shared_t {
            rusage,
            timer_first,
            status,
        } = shared.get().context("get shared error")?;
        let cpu_time = Elapse::from(rusage.ru_utime) + Elapse::from(rusage.ru_stime);
        // on macos, the maximum resident set size is measured in bytes (see man getrusage)
        #[cfg(target_os = "macos")]
        let memory = Memory::from(rusage.ru_maxrss as u64);
        #[cfg(not(target_os = "macos"))]
        let memory = Memory::from((rusage.ru_maxrss * 1024) as u64);

        macro_rules! real_tle {
            () => {
                !self.limits.real_time.check(&real_time)
            };
        }
        let child_status = signal_safe::WaitStatus(status);
        let status: Status = if child_status.exited() {
            println!("子进程正常退出");
            let exit_code = child_status.exitstatus();
            if !self.limits.real_memory.check(&memory) {
                Status::MemoryLimitExceeded
            } else if timer_first != 0 || real_tle!() {
                Status::TimeLimitExceeded
            } else if exit_code != 0 {
                Status::RuntimeError(child_status.0)
            } else {
                Status::Ok
            }
        } else {
            if !child_status.signaled() {
                return Err(anyhow::anyhow!("unknown child status {}", child_status.0));
            }
            println!("子进程被信号终止");
            let signal = child_status.termsig();
            if signal == signal_safe::SIGKILL || real_tle!() {
                println!("子进程被计时线程终止");
                Status::TimeLimitExceeded
            } else {
                Status::RuntimeError(child_status.0)
            }
        };
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
    fn exec_sandbox(&self) -> anyhow::Result<crate::Termination> {
        // prepare for arguments
        let args = crate::to_exec_array(self.arguments.clone());
        let env = crate::to_exec_array(self.envs.clone());

        let start = Instant::now(); // record the real time duration of tested process
        let guard = signal_safe::sigblockall(); // block all signals before forking
        if guard.contains(signal_safe::SIGCHLD)? {
            return Err(anyhow::anyhow!("previous block sigset contains SIGCHLD"));
        }
        let shared = share_mem::GlobalShared::init(); // should be freed in exec_parent

        std::io::stderr().flush()?;

        let r = match signal_safe::fork() {
            Ok(0) => {
                let err = self.exec_child(self.exec_path.as_c_str(), &args, &env, guard, shared);
                if let Err(err) = err {
                    signal_safe::print_cstr(b"exec_sandbox child: errno = \x00");
                    signal_safe::print_i64(err.0 as i64);
                    signal_safe::print_cstr(b"\n\x00");
                    signal_safe::exit(1);
                }
                signal_safe::exit(0);
            }
            Ok(pid) => self.exec_parent(pid, start, guard, &shared),
            Err(e) => {
                drop(guard); // recover signal mask
                Err(e.into())
            }
        };
        shared.free();
        r.context("exec_sandbox error")
    }
}

// new API
impl Singleton {
    /// Create a new builder with the path of executable
    pub fn new(exec: CString) -> Self {
        Singleton {
            limits: Limitation::default(),
            stdin: None,
            stdout: None,
            stderr: None,
            exec_path: exec,
            arguments: Vec::new(),
            envs: Vec::new(),
        }
    }
    /// set the path of input file, which will be rediected to stdin.
    pub fn stdin(mut self, arg: CString) -> Self {
        self.stdin = Some(arg);
        self
    }
    /// set the path of output file, which will be rediected to stdout.
    pub fn stdout(mut self, arg: CString) -> Self {
        self.stdout = Some(arg);
        self
    }
    /// set the path of error output file, which will be rediected to stderr.
    pub fn stderr(mut self, arg: CString) -> Self {
        self.stderr = Some(arg);
        self
    }
    /// add an argument to the end of argument list
    pub fn push_arg(mut self, args: impl IntoIterator<Item = CString>) -> Self {
        for arg in args {
            self.arguments.push(arg);
        }
        self
    }
    /// add an argument to the end of environment list
    pub fn push_env(mut self, args: impl IntoIterator<Item = CString>) -> Self {
        for arg in args {
            self.envs.push(arg);
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
    use std::os::unix::ffi::OsStrExt;
    use std::process::Command;

    use super::*;
    use crate::unix::Lim;
    use crate::ExecSandBox;

    macro_rules! cstring {
        ($e:expr) => {
            CString::new($e.as_bytes().to_vec()).unwrap()
        };
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_free() -> anyhow::Result<()> {
        let ls_path = if cfg!(target_os = "linux") {
            "/usr/bin/ls"
        } else {
            "/bin/ls"
        };

        let singleton = Singleton::new(cstring!(ls_path)).push_arg([
            cstring!("ls"),
            cstring!("-l"),
            cstring!("."),
        ]);

        let term = singleton.exec_sandbox()?;
        assert_eq!(term.status, Status::Ok);
        println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_tle_real() -> anyhow::Result<()> {
        let sleep_path = if cfg!(target_os = "linux") {
            "/usr/bin/sleep"
        } else {
            "/bin/sleep"
        };
        // sleep 5 秒，触发 TLE
        let singleton = Singleton::new(cstring!(sleep_path))
            .push_arg([cstring!("sleep"), cstring!("2")])
            .set_limits(|mut l| {
                l.cpu_time = Lim::Double(1000.into(), 3000.into());
                l.real_time = Lim::Double(1000.into(), 2000.into());
                l
            });

        let term = singleton.exec_sandbox()?;
        assert_eq!(term.status, Status::TimeLimitExceeded);
        // println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    #[cfg_attr(not(unix), ignore = "not unix os")]
    fn singleton_env() -> anyhow::Result<()> {
        let env_path = "/usr/bin/env";

        let singleton = Singleton::new(cstring!(env_path)).push_arg([
            cstring!("env"),
            cstring!("DIR=/usr"),
            cstring!("A=b"),
        ]);

        let term = singleton.exec_sandbox()?;
        assert_eq!(term.status, Status::Ok);
        // println!("termination: {:?}", term);
        Ok(())
    }

    #[test]
    fn singleton_loop() -> anyhow::Result<()> {
        let dir = tempfile::TempDir::new().unwrap();
        let main_path = dir.path().join("main.c");
        let exec_path = dir.path().join("main");
        std::fs::write(
            &main_path,
            r"
int main() {
    for(;;);
}
",
        )
        .unwrap();
        let mut p = Command::new("gcc")
            .arg(&main_path)
            .arg("-o")
            .arg(&exec_path)
            .spawn()
            .unwrap();
        let r = p.wait().unwrap();
        assert!(exec_path.is_file());
        assert!(r.success());

        let term = Singleton::new(cstring!(exec_path.as_os_str()))
            .set_limits(|mut l| {
                l.cpu_time = Lim::Double(1000.into(), 3000.into());
                l.real_time = Lim::Double(1000.into(), 2000.into());
                l
            })
            .exec_sandbox()?;
        assert_eq!(term.status, Status::TimeLimitExceeded);
        let term = Singleton::new(cstring!(exec_path.as_os_str()))
            .set_limits(|mut l| {
                l.cpu_time = Lim::Double(1000.into(), 3000.into());
                l.real_time = Lim::Double(1000.into(), 2000.into());
                l
            })
            .exec_sandbox()?;
        assert_eq!(term.status, Status::TimeLimitExceeded);

        drop(dir);
        Ok(())
    }
}
