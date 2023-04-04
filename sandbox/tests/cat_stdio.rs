#[test]
#[cfg(target_os = "linux")]
fn test_cat_stdio() -> Result<(), sandbox::UniError> {
    use std::io::Write;

    use sandbox::{sigton, ExecSandBox};
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

    let s = sigton! {
        exec: "/usr/bin/cat";
        stdin: filepath;
        stdout: outputpath;
        lim cpu_time:       6000       7000;
        lim real_time:      7000;
        lim real_memory:    2 * GB;
        lim virtual_memory: 2 * GB     3 * GB;
        lim stack:          2 * GB     3 * GB;
        lim output:         256 * MB   1 * GB;
        lim fileno:         30         30;
    };

    let term = s.exec_sandbox()?;

    assert_eq!(term.status, sandbox::Status::Ok);

    let out_str = std::fs::read_to_string(outputpath)?;

    assert_eq!(out_str, content);

    Ok(())
}
