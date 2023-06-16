#[test]
#[cfg(target_os = "linux")]
fn test_cat_stdio() -> Result<(), sandbox::SandboxError> {
    use std::io::Write;

    use sandbox::{
        unix::{Limitation, SingletonBuilder},
        Builder, ExecSandBox,
    };
    use tempfile::tempdir;

    let dir = tempdir()?;
    let filepath = &dir.path().join("input.txt");
    let outputpath = &dir.path().join("output.txt");
    let mut fin = std::fs::File::create(filepath).unwrap();

    let content = "hello\n world";
    fin.write_all(content.as_bytes())?;
    drop(fin);

    const MB: u64 = 1024 * 1024_u64;
    const GB: u64 = 1024 * MB;

    let s = SingletonBuilder::new("/usr/bin/cat")
        .stdin(filepath)
        .stdout(outputpath)
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

    let term = s.exec_sandbox()?;

    assert_eq!(term.status, sandbox::Status::Ok);

    let out_str = std::fs::read_to_string(outputpath)?;

    assert_eq!(out_str, content);

    Ok(())
}
