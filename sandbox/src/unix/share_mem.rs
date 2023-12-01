use super::signal_safe::{errno_result, Errno};

mod cbind {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/sharecc.rs"));
}

pub use cbind::{global_shared_t, rusage};

#[derive(Clone)]
pub struct GlobalShared(*mut global_shared_t);

impl GlobalShared {
    pub fn init() -> Self {
        Self(unsafe { cbind::init_shared() })
    }
    pub fn get(&self) -> Option<global_shared_t> {
        unsafe {
            if self.0 == std::ptr::null_mut() {
                None
            } else {
                Some(*self.0)
            }
        }
    }
    pub fn try_set(&self, value: global_shared_t) -> bool {
        unsafe {
            if self.0 == std::ptr::null_mut() {
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

pub fn get_rusage() -> Result<cbind::rusage, Errno> {
    unsafe {
        let mut rusage: cbind::rusage = cbind::rusage {
            ru_utime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: cbind::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        };
        let rc = cbind::getrusage(cbind::RUSAGE_CHILDREN, &mut rusage as *mut cbind::rusage);
        if rc < 0 {
            errno_result()
        } else {
            Ok(rusage)
        }
    }
}

impl From<cbind::timeval> for crate::Elapse {
    /// 单位：ms
    fn from(value: cbind::timeval) -> Self {
        Self((value.tv_sec * 1000 + value.tv_usec as i64 / 1000) as u64)
    }
}
