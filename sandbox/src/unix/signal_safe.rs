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
    unsafe { cbind::sio_error(msg.as_ptr() as *const i8) }
    unreachable!()
}

pub fn dprint(fd: i32, s: &[u8]) -> isize {
    unsafe { cbind::sio_dputs(fd as i32, s.as_ptr() as *const i8) }
}

pub fn dprint_i64(fd: i32, num: i64) -> isize {
    unsafe { cbind::sio_dputl(fd as i32, num) }
}

/// For async-signal-safety, you should use `b"..."` to declare a byte string.
/// make sure to add the \x00 for null-termination.
pub fn print_str(s: &[u8]) -> isize {
    // unsafe { cbind::sio_puts(s.as_ptr() as *const i8) }
    dprint(STDOUT_FILENO, s)
}

pub fn print_i64(num: i64) -> isize {
    dprint_i64(STDOUT_FILENO, num)
}

pub fn dup2(to: i32, from: i32) {
    unsafe {
        if cbind::dup2(to, from) < 0 {
            print_str(b"warning: dup failed\n\x00");
        }
    }
}

/// equivalent to `setpgid(0, 0)`
pub fn set_self_grp() {
    unsafe {
        if cbind::setpgid(0, 0) < 0 {
            error_exit(b"setpgid error\n\x00"); /* abort */
        }
    }
}

pub fn execve(path: &CStr, argv: &[*mut std::ffi::c_char], envp: &[*mut std::ffi::c_char]) -> ! {
    unsafe {
        if cbind::execve(path.as_ptr(), argv.as_ptr(), envp.as_ptr()) < 0 {
            error_exit(b"execve error\n\x00")
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

pub fn sigblockall() -> u32 {
    unsafe { cbind::sigblockall() }
}
pub fn sigsetmask(mask: u32) {
    unsafe {
        cbind::Sigsetmask(mask);
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

pub use cbind::RLIMIT_AS;
pub use cbind::RLIMIT_CPU;
pub use cbind::RLIMIT_FSIZE;
pub use cbind::RLIMIT_NOFILE;
pub use cbind::RLIMIT_STACK;

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

pub use cbind::{ECHILD, WCONTINUED, WEXITED, WNOHANG, WSTOPPED, WUNTRACED};
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

pub fn sigsuspend(sigmask: u32) {
    unsafe {
        let rc = cbind::sigsuspend(&sigmask as *const u32);
        if rc < 0 {
            if cbind::get_errno() as u32 == cbind::EFAULT {
                error_exit(b"sigsuspend: invalid sigmask\n\x00")
            }
        }
    }
}

pub fn sleep(sec: u32) {
    unsafe {
        cbind::sleep(sec);
    }
}

pub use cbind::{ESRCH, SIGKILL};
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

#[cfg(test)]
mod tests {
    use crate::unix::signal_safe::cbind;

    #[test]
    fn test_fork() {
        unsafe {
            let pid = cbind::fork();
            if pid < 0 {
                super::print_str(b"fork failed\n\x00");
            }
            if pid == 0 {
                super::print_str(b"child\n\x00");
                cbind::_exit(0);
            } else {
                let mut status = 0;
                let pid = cbind::wait(&mut status as *mut i32);
                super::print_str(b"parent\n\x00");
                println!("pid = {pid}");
            }
        }
        println!("Hello, world!");
    }
}
