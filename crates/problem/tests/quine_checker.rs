pub fn inner() -> Result<bool, std::io::Error> {
    let output = std::fs::read_to_string("output")?;
    // find the source file
    for entry in std::fs::read_dir(".")?.filter_map(Result::ok) {
        if !entry.file_type()?.is_file() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with("main-pre")
                && (name.ends_with(".rs")
                    || name.ends_with(".c")
                    || name.ends_with(".cpp")
                    || name.ends_with(".py")
                    || name.ends_with(".s"))
            {
                let input = std::fs::read_to_string(name)?;
                println!("source: [{input}]");
                println!("output: [{output}]");
                return Ok(input == output);
            }
        }
    }
    println!("cannot find source file");
    Ok(false)
}

/// This checker is based on the traditional problem judger. It reads from `input`
/// and `output` and then check of they're the same.
#[no_mangle]
pub extern "C" fn check() -> f32 {
    match inner() {
        Ok(eq) => {
            if eq {
                1.0
            } else {
                println!("input is not the same as output");
                0.0
            }
        }
        Err(msg) => {
            println!("{msg}");
            0.0
        }
    }
}
