use std::io::Write;
use tempfile::tempdir;

macro_rules! cstring {
    ($e:expr) => {
        std::ffi::CString::new($e.as_bytes().to_vec()).unwrap()
    };
}

#[test]
#[cfg(unix)]
#[cfg_attr(not(target_os = "linux"), ignore = "not linux")]
fn test_gcc_linux() -> anyhow::Result<()> {
    use std::os::unix::ffi::OsStrExt;

    use sandbox::{
        unix::{Lim, Limitation, Singleton},
        ExecSandBox, Status,
    };

    let dir = tempdir().unwrap();
    let filepath = &dir.path().join("main.cpp");
    let execpath = &dir.path().join("main");
    let mut file = std::fs::File::create(filepath).unwrap();
    let source = include_str!("asserts/stress.txt");
    file.write_all(source.as_bytes()).unwrap();
    let s = Singleton::new(cstring!("/usr/bin/g++"))
        .push_arg([
            cstring!("g++"),
            cstring!(filepath.as_os_str()),
            cstring!("-o"),
            cstring!(execpath.as_os_str()),
            cstring!("-O2"),
        ])
        .push_env([cstring!("PATH=/user/local/bin:/usr/bin")])
        .set_limits(|_| Limitation {
            real_time: Lim::Single(7000.into()),
            cpu_time: Lim::Single(7000.into()),
            virtual_memory: Lim::Single((2 << 30).into()),
            real_memory: Lim::Single((2 << 30).into()),
            stack_memory: Lim::Single((2 << 30).into()),
            output_memory: Lim::Single((64 << 20).into()),
            fileno: Lim::Single(30),
        });
    let term = s.exec_sandbox()?;
    assert_eq!(term.status, Status::Ok);
    dbg!(&term);
    Ok(())
}
