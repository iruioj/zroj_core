use super::Limitation;
use serde::{Deserialize, Serialize};

/// Serializable config format for singleton
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

#[cfg(feature = "exec_sandbox")]
impl From<SingletonConfig> for super::Singleton {
    fn from(value: SingletonConfig) -> Self {
        use std::ffi::CString;

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
            stdout: CString::new(value.stdout.unwrap_or("/dev/null".to_string())).unwrap(),
            stderr: CString::new(value.stderr.unwrap_or("/dev/null".to_string())).unwrap(),
        }
    }
}

// new API
impl SingletonConfig {
    /// Create a new builder with the path of executable
    pub fn new(exec: impl AsRef<str>) -> Self {
        Self {
            limits: Limitation::default(),
            stdin: None,
            stdout: None,
            stderr: None,
            exec_path: exec.as_ref().to_string(),
            arguments: Vec::new(),
            envs: Vec::new(),
        }
    }
    /// set the path of input file, which will be rediected to stdin.
    pub fn stdin(mut self, arg: impl AsRef<str>) -> Self {
        self.stdin = Some(arg.as_ref().to_string());
        self
    }
    /// set the path of output file, which will be rediected to stdout.
    pub fn stdout(mut self, arg: impl AsRef<str>) -> Self {
        self.stdout = Some(arg.as_ref().to_string());
        self
    }
    /// set the path of error output file, which will be rediected to stderr.
    pub fn stderr(mut self, arg: impl AsRef<str>) -> Self {
        self.stderr = Some(arg.as_ref().to_string());
        self
    }
    /// add an argument to the end of argument list
    pub fn push_args<'a>(mut self, args: impl IntoIterator<Item = &'a str>) -> Self {
        for arg in args {
            self.arguments.push(arg.to_string());
        }
        self
    }
    /// add an argument to the end of environment list
    pub fn push_envs<'a>(mut self, args: impl IntoIterator<Item = &'a str>) -> Self {
        for arg in args {
            self.envs.push(arg.to_string());
        }
        self
    }
    /// add current process's env to the list
    pub fn with_current_env(mut self) -> Self {
        for (key, value) in std::env::vars() {
            self.envs.push(format!("{}={}", key, value));
        }
        self
    }
    /// set resource limitation
    pub fn set_limits(mut self, modifier: impl FnOnce(Limitation) -> Limitation) -> Self {
        self.limits = modifier(self.limits);
        self
    }
    /// Build the final singleton object
    #[cfg(feature = "exec_sandbox")]
    pub fn build(self) -> super::Singleton {
        self.into()
    }
}
