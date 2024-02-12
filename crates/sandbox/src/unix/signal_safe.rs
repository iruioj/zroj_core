//! This module provides async-signal-safe utilities
//!
//! MAKE SURE all exposed functions are async-signal-safe

mod cbind {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/sigutilscc.rs"));
}

pub const STDERR_FILENO: i32 = cbind::STDERR_FILENO as i32;
pub const STDIN_FILENO: i32 = cbind::STDIN_FILENO as i32;
pub const STDOUT_FILENO: i32 = cbind::STDOUT_FILENO as i32;

use std::ffi::CStr;

/// error denoted by errno
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Errno(pub i32);

impl Errno {
    pub fn is_errno(&self, errno: u32) -> bool {
        self.0 as u32 == errno
    }
}

impl std::fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Errno {}

/// retreive errno and return it as a result
pub fn errno_result<T>() -> Result<T, Errno> {
    Err(Errno(unsafe { cbind::get_errno() }))
}

fn error_exit(msg: &[u8]) -> ! {
    unsafe { cbind::sio_error(msg.as_ptr().cast()) }
    unreachable!()
}

pub fn dprint(fd: i32, s: &[u8]) -> isize {
    debug_assert!(*s.last().unwrap() == 0);
    unsafe { cbind::sio_dputs(fd, s.as_ptr().cast()) }
}

pub fn dprint_i64(fd: i32, num: i64) -> isize {
    unsafe { cbind::sio_dputl(fd, num) }
}

/// For async-signal-safety, you should use `b"..."` to declare a byte string.
/// make sure to add the \x00 for null-termination.
pub fn print_cstr(s: &[u8]) -> isize {
    dprint(STDERR_FILENO, s)
}

pub fn print_i64(num: i64) -> isize {
    dprint_i64(STDERR_FILENO, num)
}

pub fn dup2(to: i32, from: i32) {
    unsafe {
        if cbind::dup2(to, from) < 0 {
            print_cstr(b"warning: dup failed\n\x00");
        }
    }
}

/// equivalent to `setpgid(0, 0)`
pub fn set_self_grp() {
    unsafe {
        if cbind::setpgid(0, 0) < 0 {
            error_exit(b"setpgid error\x00"); /* abort */
        }
    }
}

pub fn execve(path: &CStr, argv: &[*mut std::ffi::c_char], envp: &[*mut std::ffi::c_char]) -> ! {
    unsafe {
        if cbind::execve(path.as_ptr(), argv.as_ptr(), envp.as_ptr()) < 0 {
            error_exit(b"execve error\x00")
        }
    }
    unreachable!()
}

pub fn open_read(path: &CStr) -> Result<i32, Errno> {
    unsafe {
        let fd = cbind::open_read_file(path.as_ptr());
        if fd < 0 {
            errno_result()
        } else {
            Ok(fd)
        }
    }
}

pub fn open_write(path: &CStr) -> Result<i32, Errno> {
    unsafe {
        let fd = cbind::open_write_file(path.as_ptr());
        if fd < 0 {
            errno_result()
        } else {
            Ok(fd)
        }
    }
}

pub type Sigset = cbind::sigset_t;
pub fn sigblockall() -> SigblockGuard {
    SigblockGuard(unsafe { cbind::sigblockall() })
}
pub fn sigsetmask(mask: Sigset) {
    unsafe {
        cbind::Sigsetmask(mask);
    }
}
pub fn sigismember(mask: &Sigset, signo: u32) -> Result<bool, Errno> {
    let r = unsafe { cbind::sigismember(mask, signo as i32) };
    if r < 0 {
        errno_result()
    } else {
        Ok(r == 1)
    }
}

/// Wrap a sigset as a guard of [`sigblockall`]. Certain signals are blocked
/// during the life span of this guard.
pub struct SigblockGuard(Sigset);

impl SigblockGuard {
    pub fn contains(&self, signo: u32) -> Result<bool, Errno> {
        sigismember(&self.0, signo)
    }
    /// see [`sigsuspend`]
    pub fn suspend(&self) {
        sigsuspend(&self.0)
    }
}

impl Drop for SigblockGuard {
    fn drop(&mut self) {
        sigsetmask(self.0);
    }
}

/// wraper for syscall `fork`
pub fn fork() -> Result<i32, Errno> {
    unsafe {
        let pid = cbind::fork();
        if pid < 0 {
            errno_result()
        } else {
            Ok(pid)
        }
    }
}

#[cfg(target_os = "linux")]
pub const RLIMIT_AS: u32 = cbind::__rlimit_resource_RLIMIT_AS;
#[cfg(target_os = "linux")]
pub const RLIMIT_CPU: u32 = cbind::__rlimit_resource_RLIMIT_CPU;
#[cfg(target_os = "linux")]
pub const RLIMIT_STACK: u32 = cbind::__rlimit_resource_RLIMIT_STACK;
#[cfg(target_os = "linux")]
pub const RLIMIT_FSIZE: u32 = cbind::__rlimit_resource_RLIMIT_FSIZE;
#[cfg(target_os = "linux")]
pub const RLIMIT_NOFILE: u32 = cbind::__rlimit_resource_RLIMIT_NOFILE;

#[cfg(target_os = "macos")]
pub use {cbind::RLIMIT_FSIZE, cbind::RLIMIT_NOFILE};

pub fn setrlimit(resource: i32, rlim_cur: u64, rlim_max: u64) -> Result<(), Errno> {
    unsafe {
        let r = cbind::Setrlimit(resource, rlim_cur, rlim_max);
        if r < 0 {
            errno_result()
        } else {
            Ok(())
        }
    }
}

pub fn exit(status: i32) -> ! {
    unsafe { cbind::_exit(status) }
}

#[derive(Debug, Clone, Copy)]
pub struct WaitStatus(pub i32);
impl WaitStatus {
    /// WIFEXITED
    pub fn exited(self) -> bool {
        unsafe { cbind::wrap_WIFEXITED(self.0) > 0 }
    }
    /// WIFSIGNALED
    pub fn signaled(self) -> bool {
        unsafe { cbind::wrap_WIFSIGNALED(self.0) > 0 }
    }
    /// WEXITSTATUS
    pub fn exitstatus(self) -> i32 {
        unsafe { cbind::wrap_WEXITSTATUS(self.0) }
    }
    /// WTERMSIG
    pub fn termsig(self) -> u32 {
        unsafe { cbind::wrap_WTERMSIG(self.0) as u32 }
    }
}

pub use cbind::{ECHILD, WNOHANG};
/// return (pid, status)
pub fn waitpid(pid: i32, options: u32) -> Result<(i32, WaitStatus), Errno> {
    unsafe {
        let mut status = 0;
        let rc = cbind::waitpid(pid, &mut status as *mut i32, options as i32);
        if rc < 0 {
            errno_result()
        } else {
            Ok((rc, WaitStatus(status)))
        }
    }
}

pub fn sigsuspend(sigmask: &Sigset) {
    unsafe {
        let rc = cbind::sigsuspend(sigmask as *const Sigset);
        if rc < 0 && cbind::get_errno() as u32 == cbind::EFAULT {
            error_exit(b"sigsuspend: invalid sigmask\n\x00")
        }
    }
}

pub fn getpid() -> i32 {
    unsafe { cbind::getpid() }
}

/// register a signal handler that print the signal
pub fn signal_echo(signo: u32) {
    unsafe {
        cbind::signal_echo(signo as i32);
    }
}

pub fn sleep(sec: u32) {
    unsafe {
        cbind::sleep(sec);
    }
}

pub fn kill(pid: i32, sig: u32) -> Result<(), Errno> {
    unsafe {
        let r = cbind::kill(pid, sig as i32);
        if r < 0 {
            errno_result()
        } else {
            Ok(())
        }
    }
}

/// Using `cbind::SIGCHLD` in linux dev container on MacOS results
/// in incorrect signal number. It looks like a bug of bindgen.
pub fn get_sigchld() -> u32 {
    unsafe { cbind::get_sigchld() as u32 }
}
pub fn get_sigkill() -> u32 {
    unsafe { cbind::get_sigkill() as u32 }
}
pub fn get_sigxcpu() -> u32 {
    unsafe { cbind::get_sigxcpu() as u32 }
}

#[cfg(test)]
mod tests {
    use crate::unix::signal_safe::cbind;

    #[test]
    fn test_fork() {
        unsafe {
            let pid = cbind::fork();
            if pid < 0 {
                super::print_cstr(b"fork failed\n\x00");
            }
            if pid == 0 {
                super::print_cstr(b"child\n\x00");
                cbind::_exit(0);
            } else {
                let mut status = 0;
                let pid = cbind::wait(&mut status as *mut i32);
                super::print_cstr(b"parent\n\x00");
                println!("pid = {pid}");
            }
        }
        println!("Hello, world!");
    }
    #[test]
    fn test_signo() {
        println!("SIGCHLD = {} (should be 17 on linux)", cbind::SIGCHLD);
        println!("get_sigchld = {} (should be 17 on linux)", unsafe {
            cbind::get_sigchld()
        });
    }
}
