#[test]
#[cfg(target_os = "linux")]
fn test_cat_stdio() -> Result<(), sandbox::SandboxError> {
    use std::io::Write;

    use sandbox::{
        unix::{Limitation, SingletonBuilder, Lim},
        Builder, ExecSandBox,
    };
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let filepath = &dir.path().join("input.txt");
    let outputpath = &dir.path().join("output.txt");
    let mut fin = std::fs::File::create(filepath).unwrap();

    let content = "hello\n world";
    fin.write_all(content.as_bytes()).unwrap();
    drop(fin);

    let s = SingletonBuilder::new("/usr/bin/cat")
        .stdin(filepath)
        .stdout(outputpath)
        .set_limits(|_| Limitation {
            real_time: Lim::Single(7000.into()),
            cpu_time: Lim::Single(7000.into()),
            virtual_memory: Lim::Single((2 << 30).into()),
            real_memory: Lim::Single((2 << 30).into()),
            stack_memory: Lim::Single((2 << 30).into()),
            output_memory: Lim::Single((64 << 20).into()),
            fileno: Lim::Single(10),
        })
        .build()?;

    let term = s.exec_sandbox()?;

    assert_eq!(term.status, sandbox::Status::Ok);

    let out_str = std::fs::read_to_string(outputpath).unwrap();

    assert_eq!(out_str, content);

    Ok(())
}
