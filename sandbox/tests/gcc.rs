use std::io::Write;
use tempfile::tempdir;

#[test]
#[cfg(all(unix))]
#[cfg_attr(not(target_os = "linux"), ignore = "not linux")]
fn gcc_linux() -> Result<(), sandbox::UniError> {
    use sandbox::{sigton, ExecSandBox, Status};

    let dir = tempdir()?;
    let filepath = &dir.path().join("main.cpp");
    let execpath = &dir.path().join("main");
    let mut file = std::fs::File::create(filepath)?;
    let source = include_str!("asserts/stress.cpp");
    file.write(source.as_bytes())?;
    const MB: u64 = 1024 * 1024_u64;
    const GB: u64 = 1024 * MB;
    let s = sigton! {
        exec: "/usr/bin/g++";
        cmd: "g++" filepath "-o" execpath "-O2";
        env: "PATH=/usr/local/bin:/usr/bin";
        lim cpu_time:       6000       7000;
        lim real_time:      7000;
        lim real_memory:    2 * GB;
        lim virtual_memory: 2 * GB     3 * GB;
        lim stack:          2 * GB     3 * GB;
        lim output:         256 * MB   1 * GB;
        lim fileno:         30         30;
    };
    let term = s.exec_fork()?;
    assert_eq!(term.status, Status::Ok);
    dbg!(&term);
    Ok(())
}
