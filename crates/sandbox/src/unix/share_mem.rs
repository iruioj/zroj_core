use super::signal_safe::{errno_result, Errno};

mod cbind {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/sharecc.rs"));
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

pub fn get_rusage() -> Result<cbind::rusage_t, Errno> {
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

impl From<cbind::timeval> for crate::Elapse {
    /// 单位：ms
    fn from(value: cbind::timeval) -> Self {
        Self((value.tv_sec * 1000 + value.tv_usec as i64 / 1000) as u64)
    }
}
