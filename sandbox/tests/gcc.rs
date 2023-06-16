use std::io::Write;
use tempfile::tempdir;

#[test]
#[cfg(all(unix))]
#[cfg_attr(not(target_os = "linux"), ignore = "not linux")]
fn test_gcc_linux() -> Result<(), sandbox::SandboxError> {
    use sandbox::{
        unix::{Limitation, SingletonBuilder},
        Builder, ExecSandBox, Status,
    };

    let dir = tempdir()?;
    let filepath = &dir.path().join("main.cpp");
    let execpath = &dir.path().join("main");
    let mut file = std::fs::File::create(filepath)?;
    let source = include_str!("asserts/stress.txt");
    file.write(source.as_bytes())?;
    const MB: u64 = 1024 * 1024_u64;
    const GB: u64 = 1024 * MB;
    let s = SingletonBuilder::new("/usr/bin/g++")
        .push_arg("g++")
        .push_arg(filepath)
        .push_arg("-o")
        .push_arg(execpath)
        .push_arg("-O2")
        .push_env("PATH=/user/local/bin:/usr/bin")
        .set_limits(|_| Limitation {
            real_time: Some(7000),
            cpu_time: Some((6000, 7000)),
            virtual_memory: Some((2 * GB, 3 * GB)),
            real_memory: Some(2 * GB),
            stack_memory: Some((2 * GB, 3 * GB)),
            output_memory: Some((256 * MB, 1 * GB)),
            fileno: Some((30, 30)),
        })
        .build()?;
    let term = s.exec_fork()?;
    assert_eq!(term.status, Status::Ok);
    dbg!(&term);
    Ok(())
}
