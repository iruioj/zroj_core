use super::sigsafe::{errno_result, Errno, WaitStatus};

mod cbind {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/utilscc.rs"));
}

pub use cbind::global_shared_t;

#[derive(Clone)]
pub struct GlobalShared(*mut global_shared_t);

impl GlobalShared {
    pub fn init() -> Self {
        Self(unsafe { cbind::init_shared() })
    }
    pub fn get(&self) -> Option<global_shared_t> {
        unsafe {
            if self.0.is_null() {
                None
            } else {
                Some(*self.0)
            }
        }
    }
    pub fn try_set(&self, value: global_shared_t) -> bool {
        unsafe {
            if self.0.is_null() {
                false
            } else {
                *self.0 = value;
                true
            }
        }
    }
    pub fn free(&self) {
        unsafe {
            cbind::free_shared(self.0);
        }
    }
}

pub fn _get_rusage_children() -> Result<cbind::rusage_t, Errno> {
    unsafe {
        let mut rusage = cbind::rusage_t {
            ru_utime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
        };
        let rc = cbind::get_children_rusage(&mut rusage as *mut cbind::rusage_t);
        if rc < 0 {
            errno_result()
        } else {
            Ok(rusage)
        }
    }
}
pub fn get_rusage_self() -> Result<cbind::rusage_t, Errno> {
    unsafe {
        let mut rusage = cbind::rusage_t {
            ru_utime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
        };
        let rc = cbind::get_self_rusage(&mut rusage as *mut cbind::rusage_t);
        if rc < 0 {
            errno_result()
        } else {
            Ok(rusage)
        }
    }
}

/// return (pid, status, rusage_t)
pub fn wait_rusage(pid: i32, options: u32) -> Result<(i32, WaitStatus, cbind::rusage_t), Errno> {
    unsafe {
        let mut rusage = cbind::rusage_t {
            ru_utime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
        };
        let mut status = 0;
        let rc = cbind::wait_rusage(
            pid,
            &mut status as *mut i32,
            options as i32,
            &mut rusage as *mut cbind::rusage_t,
        );
        if rc < 0 {
            errno_result()
        } else {
            Ok((rc, WaitStatus(status), rusage))
        }
    }
}

impl From<cbind::timeval> for crate::Elapse {
    /// 单位：ms
    fn from(value: cbind::timeval) -> Self {
        #[cfg(target_os = "linux")]
        let r = Self((value.tv_sec * 1000 + value.tv_usec / 1000) as u64);
        #[cfg(target_os = "macos")]
        let r = Self((value.tv_sec * 1000 + value.tv_usec as i64 / 1000) as u64);
        r
    }
}
