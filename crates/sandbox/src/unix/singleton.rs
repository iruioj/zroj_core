#[allow(unused)]
macro_rules! println {
    ($($rest:tt)*) => {
        compile_error!("println! is disabled in this module, use seprintln! instead")
    };
}
#[allow(unused)]
macro_rules! print {
    ($($rest:tt)*) => {
        compile_error!("print! is disabled in this module, use seprint! instead")
    };
}

use crate::{
    unix::{share_mem, sigsafe, Limitation},
    Elapse, Memory, Status, Termination,
};
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{CStr, CString},
    io::Write,
    time::Instant,
};

/// 执行单个可执行文件
#[derive(Debug, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Singleton {
    pub limits: Limitation,
    pub exec_path: CString,
    pub arguments: Vec<CString>,
    pub envs: Vec<CString>,
    /// 为 None 表示不提供/不获取 读入/输出
    pub stdin: Option<CString>,
    // to avoid messing up with the output of calling process, stdout and stderr are required to provided,
    // otherwise they are set to `/dev/null`.
    pub stdout: CString,
    pub stderr: CString,
}

impl Singleton {
    /// In child process, before calling execve, you should only execute
    /// async-signal-safe funtions, thus don't use unistd.
    fn exec_child(
        &self,
        path: &CStr,
        args: &[*mut std::ffi::c_char],
        env: &[*mut std::ffi::c_char],
        guard: sigsafe::SigblockGuard,
        shared: share_mem::GlobalShared,
    ) -> Result<(), sigsafe::Errno> {
        // register a handler for SIGCHLD to make sigsuspend work
        sigsafe::signal_echo(sigsafe::get_sigchld());

        let max_rss_before = share_mem::get_rusage_self()
            .map(|o| o.ru_maxrss)
            .unwrap_or(0);

        seprintln!("(child) self's max_rss before execve: {max_rss_before}");

        // fork another child process to execute program
        // at this time, all signals are blocked, so we can fork directly
        let pid_child = sigsafe::fork()?;
        if pid_child == 0 {
            seprintln!("(child-child) pid = {}", sigsafe::getpid());
            sigsafe::set_self_grp();

            // set resource limit
            macro_rules! setlim {
                ($i:ident, $r:ident, $f:ident) => {
                    match self.limits.$i {
                        super::Lim::Single(s) => {
                            sigsafe::setrlimit(sigsafe::$r as i32, s.$f(), s.$f())?;
                        }
                        super::Lim::Double(s, h) => {
                            sigsafe::setrlimit(sigsafe::$r as i32, s.$f(), h.$f())?;
                        }
                    }
                };
            }
            // https://issues.chromium.org/issues/40581251#comment3
            // https://issues.fast-downward.org/issue825
            #[cfg(not(target_os = "macos"))]
            setlim!(cpu_time, RLIMIT_CPU, sec);
            #[cfg(not(target_os = "macos"))]
            setlim!(virtual_memory, RLIMIT_AS, byte);
            #[cfg(not(target_os = "macos"))]
            setlim!(stack_memory, RLIMIT_STACK, byte);
            setlim!(output_memory, RLIMIT_FSIZE, byte);
            setlim!(fileno, RLIMIT_NOFILE, into);
            seprintln!("(child-child) resource limited");

            // redirect standard IO
            if let Some(stdin) = &self.stdin {
                let fd = sigsafe::open_read(stdin)?;
                sigsafe::dup2(fd, sigsafe::STDIN_FILENO);
            }
            let fd = sigsafe::open_write(&self.stdout)?;
            sigsafe::dup2(fd, sigsafe::STDOUT_FILENO);
            let fd = sigsafe::open_write(&self.stderr)?;
            sigsafe::dup2(fd, sigsafe::STDERR_FILENO);

            drop(guard); // unblock signals
                         // todo: set syscall limit
            sigsafe::execve(path, args, env);
        }

        seprintln!("(child) fork a timmer");

        // at this time, all signals are blocked, so we can fork directly
        let pid_timer = match self.limits.real_time {
            super::Lim::Single(s) | super::Lim::Double(s, _) => {
                // fork a process to setup timer
                let pid = sigsafe::fork()?;
                if pid == 0 {
                    let secs = s.sec().clamp(0, u32::MAX as u64) as u32;
                    seprintln!("(child-timer) sleep for {secs} seconds");
                    sigsafe::sleep(secs);
                    seprintln!("(child-timer) timmer exit");
                    sigsafe::exit(0);
                }
                Some(pid)
            }
        };

        // can be interrupted either by timer or child process
        let mut timer_first = false;
        let mut child_status = None;
        let mut child_rusage = None;

        seprintln!("(child) wait for tested process and timer");

        let ru = 'outer: loop {
            // notice that sigsuspend only interrupts for signals whose action is
            // either calling handler function or exit (thus sometimes you need to
            // register handler for a signal to make it work).
            guard.suspend();
            seprintln!("(child) suspend over");
            loop {
                // since all signals are blocked, SIGCHLD will not interrupt
                let r = share_mem::wait_rusage(-1, sigsafe::WNOHANG);

                match r {
                    Ok((pid, status, ru)) => {
                        if pid_timer.is_some_and(|pid_timer| pid_timer == pid) {
                            // normally the timer should be killed by signal. thus if timer returned first -> TLE
                            if status.exited() {
                                timer_first = true;
                                // at this time, child hasn't been reaped, thus it's pid is not freed
                                if let Err(e) = sigsafe::kill(pid_child, sigsafe::get_sigkill()) {
                                    seprintln!("(child) timer kill child failed");
                                    break 'outer Err(e);
                                }
                            } // otherwise timer is killed, ignored
                        } else if pid_child == pid {
                            child_status = Some(status);
                            child_rusage = Some(ru);
                            if !timer_first {
                                // child return first
                                // at this time, timer hasn't been reaped, thus it's pid is not freed
                                if let Some(pid_timer) = pid_timer {
                                    if let Err(e) = sigsafe::kill(pid_timer, sigsafe::get_sigkill())
                                    {
                                        seprintln!("(child) kill timer failed");
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
                        if errno.is_errno(sigsafe::ECHILD) {
                            break 'outer if let Some(mut ru) = child_rusage {
                                seprintln!("(child) max_rss after execve: {}", ru.ru_maxrss);

                                // FIXME: substract the residual set size before execve.
                                // This implemention is not guaranteed to work properly, but it indead
                                // solves the increasing rusage problem. However, it also causes a
                                // decreasing rusage in the test of problem judger. Further fixes are
                                // required.
                                ru.ru_maxrss -= max_rss_before;
                                Ok(ru)
                            } else {
                                seprintln!("(child) get child rusage failed");
                                Err(super::sigsafe::Errno(1))
                            };
                        } else {
                            seprintln!("(child) child wait child failed");
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
            seprintln!("(child) set shared memory error");
            sigsafe::exit(1);
        }
        Ok(())
    }
    fn exec_parent(
        &self,
        child: i32,
        start: Instant,
        guard: sigsafe::SigblockGuard,
        shared: &share_mem::GlobalShared,
    ) -> anyhow::Result<Termination> {
        // do something before waiting for child process
        drop(guard);

        seprintln!("(parent) wait for child process id: {child}");

        let (_, child_status) = sigsafe::waitpid(child, 0).context("parent wait child error")?;

        seprintln!("(parent) wait done.");

        let real_time = start.elapsed().into();

        if !(child_status.exited() && child_status.exitstatus() == 0) {
            bail!(
                "child (parent) not normally terminated, with status {}",
                child_status.0
            );
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
        let child_status = sigsafe::WaitStatus(status);
        let status: Status = if child_status.exited() {
            seprintln!("子进程正常退出, exit_code = {}", child_status.exitstatus());
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
            seprintln!("子进程被信号终止, signal = {}", child_status.termsig());
            let signal = child_status.termsig();
            if signal == sigsafe::get_sigkill() || signal == sigsafe::get_sigxcpu() || real_tle!() {
                seprintln!("子进程被计时线程终止");
                Status::TimeLimitExceeded
            } else {
                Status::RuntimeError(child_status.0)
            }
        };
        seprintln!("主进程正常结束");
        Ok(Termination {
            status,
            real_time,
            cpu_time,
            memory,
        })
    }
}

#[cfg(feature = "exec_sandbox")]
impl crate::ExecSandBox for Singleton {
    fn exec_sandbox(&self) -> anyhow::Result<crate::Termination> {
        // flush rust codes' outputs
        std::io::stdout().flush()?;
        std::io::stderr().flush()?;

        seprintln!(
            "(parent) exec: {:?} {:?} {{ stdin: {:?}, stdout: {:?}, stderr: {:?} }}",
            self.exec_path,
            self.arguments,
            self.stdin,
            self.stdout,
            self.stderr
        );
        seprintln!("(parent) pid: {}", sigsafe::getpid());

        // prepare for arguments
        let args = crate::to_exec_array(self.arguments.clone());
        let env = crate::to_exec_array(self.envs.clone());

        let start = Instant::now(); // record the real time duration of tested process

        let guard = sigsafe::sigblockall(); // block all signals before forking
        if guard.contains(sigsafe::get_sigchld())? {
            bail!("previous block sigset contains SIGCHLD");
        }

        let shared = share_mem::GlobalShared::init(); // should be freed in exec_parent

        let r = match sigsafe::fork() {
            Ok(0) => {
                let err = self.exec_child(self.exec_path.as_c_str(), &args, &env, guard, shared);
                if let Err(err) = err {
                    seprintln!("(child) errno = {}", err);
                    sigsafe::exit(1);
                }
                sigsafe::exit(0);
            }
            Ok(pid) => self.exec_parent(pid, start, guard, &shared),
            Err(e) => Err(e.into()),
        };
        shared.free();
        r.context("exec_sandbox error")
    }
}
