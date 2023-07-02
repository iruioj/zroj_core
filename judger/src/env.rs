//! Environment.
//!
//! 检查当前评测的环境（系统、相关编译器版本、当前目录等等）

use std::path::PathBuf;

use crate::Error;

/// return "windows" or "unix".
#[allow(dead_code)]
pub fn os_family() -> &'static str {
    if cfg!(all(unix)) {
        "unix"
    } else {
        "windows"
    }
}
/// 注意，我们希望 judger 执行的命令都是我们已知的命令，
/// 而不考虑用户自定义命令（不安全），因此使用 string literal
/// 作为参数。此外命令名字里不应当包含路径（/或者\）
pub fn which(cmd_name: &'static str) -> Result<PathBuf, Error> {
    let os_cmd_name = std::ffi::OsString::from(cmd_name);
    if let Ok(path) = std::env::var("PATH") {
        if cfg!(all(unix)) {
            for p in path.split(':') {
                // eprintln!("path = {}", p);
                for entry in match std::fs::read_dir(p) {
                    Ok(r) => r,
                    Err(_) => continue,
                } {
                    let file = match entry {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    if let Ok(r) = file.file_type() {
                        if file.file_name() == os_cmd_name {
                            if r.is_symlink() {
                                return Ok(file.path());
                                // return Err(Error::CmdSymLink);
                            }
                            if r.is_file() {
                                return Ok(file.path());
                            }
                        }
                    }
                }
            }
        } else {
            todo!()
        }
    }
    Err(Error::CmdNotFound)
}

#[cfg(test)]
mod tests {
    use crate::env::os_family;
    use crate::env::which;

    #[test]
    #[cfg(all(unix))]
    fn test_linux() {
        assert_eq!(os_family(), "unix");
        eprintln!("gcc = {:?}", which("gcc"));
        eprintln!("python = {:?}", which("python"));
        eprintln!("python3 = {:?}", which("python3"));
    }
}
