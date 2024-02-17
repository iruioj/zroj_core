use std::ffi::CString;

use sandbox::unix::{Limitation, Singleton};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SingletonConfig {
    limits: Limitation,
    exec_path: String,
    arguments: Vec<String>,
    envs: Vec<String>,
    stdin: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl From<Singleton> for SingletonConfig {
    fn from(value: Singleton) -> Self {
        Self {
            limits: value.limits,
            exec_path: value.exec_path.to_str().unwrap().to_string(),
            arguments: value
                .arguments
                .into_iter()
                .map(|s| s.to_str().unwrap().to_string())
                .collect(),
            envs: value
                .envs
                .into_iter()
                .map(|s| s.to_str().unwrap().to_string())
                .collect(),
            stdin: value.stdin.map(|s| s.to_str().unwrap().to_string()),
            stdout: value.stdout.map(|s| s.to_str().unwrap().to_string()),
            stderr: value.stderr.map(|s| s.to_str().unwrap().to_string()),
        }
    }
}

impl From<SingletonConfig> for Singleton {
    fn from(value: SingletonConfig) -> Self {
        Self {
            limits: value.limits,
            exec_path: CString::new(value.exec_path).unwrap(),
            arguments: value
                .arguments
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect(),
            envs: value
                .envs
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect(),
            stdin: value.stdin.map(|s| CString::new(s).unwrap()),
            stdout: value.stdout.map(|s| CString::new(s).unwrap()),
            stderr: value.stderr.map(|s| CString::new(s).unwrap()),
        }
    }
}
