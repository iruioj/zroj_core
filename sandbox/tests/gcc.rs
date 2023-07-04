use std::io::Write;
use tempfile::tempdir;

#[test]
#[cfg(all(unix))]
#[cfg_attr(not(target_os = "linux"), ignore = "not linux")]
fn test_gcc_linux() -> Result<(), sandbox::SandboxError> {
    use sandbox::{
        unix::{Limitation, SingletonBuilder, Lim},
        Builder, ExecSandBox, Status,
    };

    let dir = tempdir().unwrap();
    let filepath = &dir.path().join("main.cpp");
    let execpath = &dir.path().join("main");
    let mut file = std::fs::File::create(filepath).unwrap();
    let source = include_str!("asserts/stress.txt");
    file.write_all(source.as_bytes()).unwrap();
    let s = SingletonBuilder::new("/usr/bin/g++")
        .push_arg("g++")
        .push_arg(filepath)
        .push_arg("-o")
        .push_arg(execpath)
        .push_arg("-O2")
        .push_env("PATH=/user/local/bin:/usr/bin")
        .set_limits(|_| Limitation {
            real_time: Lim::Single(7000.into()),
            cpu_time: Lim::Single(7000.into()),
            virtual_memory: Lim::Single((2 << 30).into()),
            real_memory: Lim::Single((2 << 30).into()),
            stack_memory: Lim::Single((2 << 30).into()),
            output_memory: Lim::Single((64 << 20).into()),
            fileno: Lim::Single(30),
        })
        .build()?;
    let term = s.exec_fork()?;
    assert_eq!(term.status, Status::Ok);
    dbg!(&term);
    Ok(())
}
