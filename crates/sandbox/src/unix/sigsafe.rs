//! This module provides async-signal-safe utilities
//!
//! MAKE SURE all exposed functions are async-signal-safe

mod cbind {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/utilscc.rs"));
}

pub const STDERR_FILENO: i32 = cbind::STDERR_FILENO as i32;
pub const STDIN_FILENO: i32 = cbind::STDIN_FILENO as i32;
pub const STDOUT_FILENO: i32 = cbind::STDOUT_FILENO as i32;

/// As [`std::fmt`] says, formatting macros avoid immediate memory allocation,
/// thus it preserves async-signal-safety.
pub struct FilenoWriter(i32);

impl core::fmt::Write for FilenoWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // If count is zero and fd refers to a regular file, then write() may return a failure status
        if s.is_empty() {
            return Ok(());
        }
        let count: isize = unsafe { cbind::write(self.0, s.as_bytes().as_ptr().cast(), s.len()) };
        if count == -1 {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub fn thread_unsafe_writer(fileno: i32) -> FilenoWriter {
    FilenoWriter(fileno)
}

/// Print formatted items to stderr with async-signal-safety
#[macro_export]
macro_rules! seprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::unix::sigsafe;
        let _ = write!(sigsafe::thread_unsafe_writer(sigsafe::STDERR_FILENO), $($arg)*);
    }}
}

/// Print formatted items to stderr with async-signal-safety
#[macro_export]
macro_rules! seprintln {
    () => {
        seprint!("\n");
    };
    ($($arg:tt)*) => {{
        seprint!($($arg)*);
        seprint!("\n");
    }};
}

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
        let name = match self.0 {
            1 => "EPERM(1): Operation not permitted",
            2 => "ENOENT(2): No such file or directory",
            3 => "ESRCH(3): No such process",
            4 => "EINTR(4): Interrupted system call",
            5 => "EIO(5): I/O error",
            6 => "ENXIO(6): No such device or address",
            7 => "E2BIG(7): Argument list too long",
            8 => "ENOEXEC(8): Exec format error",
            9 => "EBADF(9): Bad file number",
            10 => "ECHILD(10): No child processes",
            11 => "EAGAIN(11): Try again",
            12 => "ENOMEM(12): Out of memory",
            13 => "EACCES(13): Permission denied",
            14 => "EFAULT(14): Bad address",
            15 => "ENOTBLK(15): Block device required",
            16 => "EBUSY(16): Device or resource busy",
            17 => "EEXIST(17): File exists",
            18 => "EXDEV(18): Cross-device link",
            19 => "ENODEV(19): No such device",
            20 => "ENOTDIR(20): Not a directory",
            21 => "EISDIR(21): Is a directory",
            22 => "EINVAL(22): Invalid argument",
            23 => "ENFILE(23): File table overflow",
            24 => "EMFILE(24): Too many open files",
            25 => "ENOTTY(25): Not a typewriter",
            26 => "ETXTBSY(26): Text file busy",
            27 => "EFBIG(27): File too large",
            28 => "ENOSPC(28): No space left on device",
            29 => "ESPIPE(29): Illegal seek",
            30 => "EROFS(30): Read-only file system",
            31 => "EMLINK(31): Too many links",
            32 => "EPIPE(32): Broken pipe",
            33 => "EDOM(33): Math argument out of domain of func",
            34 => "ERANGE(34): Math result not representable",
            _ => "Unknown Errno",
        };
        f.write_str(name)
    }
}

impl std::error::Error for Errno {}

/// retreive errno and return it as a result
pub fn errno_result<T>() -> Result<T, Errno> {
    Err(Errno(unsafe { cbind::get_errno() }))
}

pub fn dup2(to: i32, from: i32) {
    unsafe {
        if cbind::dup2(to, from) < 0 {
            seprintln!("warning: dup failed");
        }
    }
}

/// equivalent to `setpgid(0, 0)`
pub fn set_self_grp() {
    unsafe {
        if cbind::setpgid(0, 0) < 0 {
            seprintln!("setpgid error");
            cbind::_exit(1)
        }
    }
}

pub fn execve(path: &CStr, argv: &[*mut std::ffi::c_char], envp: &[*mut std::ffi::c_char]) -> ! {
    unsafe {
        if cbind::execve(path.as_ptr(), argv.as_ptr(), envp.as_ptr()) < 0 {
            seprintln!("execve error");
            cbind::_exit(1)
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
            seprintln!("sigsuspend: invalid sigmask");
            cbind::_exit(1)
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
    use crate::unix::sigsafe::cbind;

    #[test]
    fn test_fork() {
        unsafe {
            let pid = cbind::fork();
            if pid < 0 {
                seprintln!("fork failed");
            }
            if pid == 0 {
                seprintln!("child");
                cbind::_exit(0);
            } else {
                let mut status = 0;
                let pid = cbind::wait(&mut status as *mut i32);
                seprintln!("parent");
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
