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
use std::{ffi::CString, time::Instant};

/// 执行单个可执行文件
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
#[cfg(test)]
mod tests {
    use crate::TimeLimitExceededKind;
    use crate::ExecSandBox;
    use super::*;

    #[test]
    fn singleton_free() -> Result<(), error::Error> {
        let singleton = Singleton {
            limits: vec![],
            exec_path: "/usr/bin/ls".to_string(),
            arguments: vec!["/usr/bin/ls".to_string(), "/home/sshwy".to_string()],
            envs: vec![],
        };
        let term = singleton.exec_fork()?;
        assert_eq!(term.status, Status::Ok);
        println!("termination: {:?}", term);
        Ok(())
    }
    #[test]
    fn singleton_tle_real() -> Result<(), error::Error> {
        // sleep 5 秒，触发 TLE
        // sleep 不会占用 CPU，因此触发 real time tle
        let singleton = Singleton {
            limits: vec![Limitation::CpuTime(1000, 3000), Limitation::RealTime(1000)],
            exec_path: "/usr/bin/sleep".to_string(),
            arguments: vec!["sleep".to_string(), "2".to_string()],
            envs: vec![],
        };
        let term = singleton.exec_fork()?;
        assert_eq!(
            term.status,
            Status::TimeLimitExceeded(TimeLimitExceededKind::Real)
        );
        println!("termination: {:?}", term);
        Ok(())
    }
}
