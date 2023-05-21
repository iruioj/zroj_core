//! Windows 系统下运行的沙盒
//! 不考虑安全性（本地评测用）

use serde_derive::{Deserialize, Serialize};
use winapi::shared::winerror::WAIT_TIMEOUT;

use crate::ExecSandBox;
use winapi::shared::minwindef::{DWORD};

use crate::{
    MemoryLimitExceededKind, Status, Termination, TimeLimitExceededKind,
};

extern crate winapi;

use std::os::windows::prelude::AsRawHandle;
use std::ptr;
use std::mem;
use std::process::{Command};
use std::time::{Instant};
use winapi::shared::{minwindef::*};
use winapi::um::{synchapi::*, processthreadsapi::*, jobapi2::*, winbase::*, winnt::*,psapi::*};

#[derive(Serialize, Deserialize, Debug)]
pub struct Limitation {
    /// 限制实际运行时间，一般是用来做一个大保底
    pub real_time: Option<u64>,

    /// 程序执行完后才统计内存占用情况 （byte）
    pub real_memory: Option<u64>,

    /// byte
    ///
    /// soft limit 和 hard limit，一般以 soft 为衡量标准
    pub output_memory: Option<u64>,
}

#[derive(Debug)]
pub struct Singleton {
    limits: Limitation,
    exec_path: String,
    arguments: Vec<String>,
    // envs: Vec<String>,
}

// ref1: https://docs.rs/tasklist/latest/tasklist/index.html
// ref2: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Memory/index.html?search=CreateProcess
// https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/fn.CreateProcessA.html
// https://learn.microsoft.com/zh-cn/windows/win32/procthread/creating-a-child-process-with-redirected-input-and-output?redirectedfrom=MSDN
// https://learn.microsoft.com/zh-cn/windows/win32/procthread/creating-processes
impl ExecSandBox for Singleton {
    fn exec_sandbox(&self) -> Result<crate::Termination, crate::error::Error> {
        let job = unsafe {
            let job_handle = CreateJobObjectW(ptr::null_mut(), ptr::null());
            let job = job_handle;
    
            // Set the memory limit
            
            let mem_limit = self.limits.real_memory.unwrap();
            let mut job_info = mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
            job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_JOB_MEMORY;
            job_info.JobMemoryLimit = mem_limit as usize; 
            let ret = SetInformationJobObject(job, JobObjectExtendedLimitInformation, &mut job_info as *mut _ as LPVOID, mem::size_of_val(&job_info) as u32);
            if ret == 0 {
                panic!("Failed to set job object limit: {:?}", std::io::Error::last_os_error());
            }

            job
        };
        
        let start = Instant::now();

        //println!("pwd: {}", std::env::current_dir().unwrap().display());

        let mut cmd = Command::new(self.exec_path.clone());

        for ele in &self.arguments {
            cmd.arg(ele);
        }

        // cmd.stdout(Stdio::null()).stderr(Stdio::null());
        let mut child = cmd.spawn().unwrap();   
        let pid = child.id();
        let ret = unsafe { AssignProcessToJobObject(job, child.as_raw_handle()) };
        if ret == 0 {
            panic!("Failed to assign process to job object: {:?}", std::io::Error::last_os_error());
        }

        let time_limit = self.limits.real_time.unwrap();
        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };
                
        let mut counters_ex = PROCESS_MEMORY_COUNTERS_EX {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            PageFaultCount: 0,
            PeakWorkingSetSize: 0,
            WorkingSetSize: 0,
            QuotaPeakPagedPoolUsage: 0,
            QuotaPagedPoolUsage: 0,
            QuotaPeakNonPagedPoolUsage: 0,
            QuotaNonPagedPoolUsage: 0,
            PagefileUsage: 0,
            PeakPagefileUsage: 0,
            PrivateUsage: 0,
        };

        loop {
            unsafe{
                GetProcessMemoryInfo(handle, &mut counters_ex as *mut _ as *mut _, std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32);
            }
            let wait_result = unsafe { WaitForSingleObject(child.as_raw_handle(), time_limit as DWORD) };
            match wait_result {
                WAIT_OBJECT_0 => {
                    
                    // Process has exited
                    //let process_handle: HANDLE = unsafe { OpenProcess(0x0400, FALSE, pid as u32) };
                    let status = child.wait().unwrap();
                    let duration = start.elapsed();
                    println!("Process exited with code {} after {:?}", status.code().unwrap_or(0), duration);
                    println!("{}",status);
                    

                    match status.code().unwrap() as u32 {

                        // MLE
                        STATUS_STACK_OVERFLOW => {
                            return Ok(Termination {
                                status: Status::MemoryLimitExceeded(MemoryLimitExceededKind::Real),
                                real_time: duration.as_millis() as u64,
                                cpu_time: duration.as_millis() as u64,
                                memory: counters_ex.PrivateUsage as u64,
                            });
                        }
                        // exit 0
                        0u32 => {
                            return Ok(Termination {
                                status: Status::Ok,
                                real_time: duration.as_millis() as u64,
                                cpu_time: duration.as_millis() as u64,
                                memory: counters_ex.PrivateUsage as u64,
                            });
                        }
                        // RE
                        _ => {
                            return Ok(Termination {
                                status: Status::RuntimeError(status.code().unwrap(), None),
                                real_time: duration.as_millis() as u64,
                                cpu_time: duration.as_millis() as u64,
                                memory: counters_ex.PrivateUsage as u64,
                            });
                        }
                    }
                },
                // TLE
                WAIT_TIMEOUT => {
                    let ret = unsafe { TerminateProcess(child.as_raw_handle(), 1) };
                    if ret == 0 {
                        panic!("Failed to terminate process: {:?}", std::io::Error::last_os_error());
                    }
                    let status = child.wait().unwrap();
                    let duration = start.elapsed();
                    println!("{}",status);
                    println!("Process terminated due to time limit");
                        
                    return Ok(Termination {
                        status: Status::TimeLimitExceeded(TimeLimitExceededKind::Real),
                        real_time: duration.as_millis() as u64,
                        cpu_time: duration.as_millis() as u64,
                        memory: counters_ex.PrivateUsage as u64,
                    });
                },
                _ => {
                    
                    panic!("WaitForSingleObject failed: {:?}", std::io::Error::last_os_error());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExecSandBox, windows::Limitation};

    use super::Singleton;

    #[test]
    fn test_sig() {
        // let s = Singleton {
        //     limits: Limitation {
        //         real_time: Some(1000),
        //         real_memory: Some(500 * 1024 * 1024),
        //         output_memory: Some(10 * 1024 * 1024),
        //     },
        //     exec_path: r#"C:\Program Files (x86)\Embarcadero\Dev-Cpp\TDM-GCC-64\bin\g++.exe"#.to_string(),
        //     arguments: vec!["a.cpp".to_string(), "-o".to_string(), r#"C:\Users\lyuu\zroj_core\sandbox\a.exe"#.to_string()]
        // };
        // let term = s.exec_sandbox().unwrap();
        // dbg!(term);
        let s = Singleton {
            limits: Limitation {
                real_time: Some(1000),
                real_memory: Some(100 * 1024 * 1024),
                output_memory: Some(10 * 1024 * 1024),
            },
            exec_path: r#"C:\Users\lyuu\zroj_core\sandbox\windows_test.exe"#.to_string(),
            arguments: vec![]
        };
        let term = s.exec_sandbox().unwrap();
        dbg!(term);
    }
}