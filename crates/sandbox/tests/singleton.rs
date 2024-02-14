use anyhow::Context;
use sandbox::{
    unix::{Lim, Limitation, Singleton},
    ExecSandBox, Status,
};
use std::{io::Write, process::Command};
use std::{os::unix::ffi::OsStrExt, path::PathBuf, str::FromStr};
use tempfile::tempdir;

macro_rules! cstring {
    ($e:expr) => {
        std::ffi::CString::new($e.as_bytes().to_vec()).unwrap()
    };
}

fn get_exec_path(name: &str) -> PathBuf {
    let r = Command::new("which")
        .arg(name)
        .output()
        .expect("execute which error");
    PathBuf::from_str(
        String::from_utf8(r.stdout)
            .expect("decode utf8 error")
            .trim(),
    )
    .unwrap()
}

#[test]
fn test_ls() -> anyhow::Result<()> {
    let ls_path = get_exec_path("ls");

    let singleton =
        Singleton::new(&ls_path).push_arg([cstring!("ls"), cstring!("-l"), cstring!(".")]);

    let term = singleton.exec_sandbox()?;
    assert_eq!(term.status, Status::Ok);
    println!("termination: {:?}", term);
    Ok(())
}

#[test]
fn test_sleep_tle() -> anyhow::Result<()> {
    let sleep_path = get_exec_path("sleep");
    // sleep 5 秒，触发 TLE
    let singleton = Singleton::new(&sleep_path)
        .push_arg([cstring!("sleep"), cstring!("2")])
        .set_limits(|mut l| {
            l.cpu_time = Lim::Double(1000.into(), 3000.into());
            l.real_time = Lim::Double(1000.into(), 2000.into());
            l
        });

    let term = singleton.exec_sandbox()?;
    assert_eq!(term.status, Status::TimeLimitExceeded);
    // println!("termination: {:?}", term);
    Ok(())
}

#[test]
fn test_env() -> anyhow::Result<()> {
    let env_path = get_exec_path("env");

    let singleton = Singleton::new(&env_path).push_arg([
        cstring!("env"),
        cstring!("DIR=/usr"),
        cstring!("A=b"),
    ]);

    let term = singleton.exec_sandbox()?;
    assert_eq!(term.status, Status::Ok);
    // println!("termination: {:?}", term);
    Ok(())
}

#[test]
fn test_loop() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new().unwrap();
    let main_path = dir.path().join("main.c");
    let exec_path = dir.path().join("main");
    std::fs::write(&main_path, r"int main() { for(;;); }").unwrap();
    let mut p = Command::new("gcc")
        .arg(&main_path)
        .arg("-o")
        .arg(&exec_path)
        .spawn()
        .unwrap();
    let r = p.wait().unwrap();
    assert!(exec_path.is_file() && r.success());

    let term = Singleton::new(&exec_path)
        .set_limits(|mut l| {
            l.cpu_time = Lim::Double(1000.into(), 3000.into());
            l.real_time = Lim::Double(1000.into(), 2000.into());
            l
        })
        .exec_sandbox()
        .context("the first time")?;
    assert_eq!(term.status, Status::TimeLimitExceeded);

    let term = Singleton::new(&exec_path)
        .set_limits(|mut l| {
            l.cpu_time = Lim::Double(1000.into(), 3000.into());
            l.real_time = Lim::Double(1000.into(), 2000.into());
            l
        })
        .exec_sandbox()
        .context("the second time")?;
    assert_eq!(term.status, Status::TimeLimitExceeded);

    drop(dir);
    Ok(())
}

#[test]
fn test_cat_stdio() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    let filepath = &dir.path().join("input.txt");
    let outputpath = &dir.path().join("output.txt");
    let mut fin = std::fs::File::create(filepath).unwrap();

    let content = "hello\n world";
    fin.write_all(content.as_bytes()).unwrap();
    drop(fin);

    let s = Singleton::new(&get_exec_path("cat"))
        .push_arg([cstring!("cat")])
        .stdin(cstring!(filepath.as_os_str()))
        .stdout(cstring!(outputpath.as_os_str()))
        .set_limits(|_| Limitation {
            real_time: Lim::Single(7000.into()),
            cpu_time: Lim::Single(7000.into()),
            virtual_memory: Lim::Single((2 << 30).into()),
            real_memory: Lim::Single((2 << 30).into()),
            stack_memory: Lim::Single((2 << 30).into()),
            output_memory: Lim::Single((64 << 20).into()),
            fileno: Lim::Single(10),
        });

    let term = s.exec_sandbox()?;

    assert_eq!(term.status, sandbox::Status::Ok);

    let out_str = std::fs::read_to_string(outputpath).unwrap();

    assert_eq!(out_str, content);

    Ok(())
}

#[test]
#[cfg(target_os = "linux")]
fn test_gcc_linux() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    let filepath = &dir.path().join("main.cpp");
    let execpath = &dir.path().join("main");
    let mut file = std::fs::File::create(filepath).unwrap();
    let source = include_str!("asserts/stress.txt");
    file.write_all(source.as_bytes()).unwrap();
    let s = Singleton::new(&PathBuf::from_str("/usr/bin/g++").unwrap())
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
