#[cfg(test)]
mod one_off {
    use std::{fs::File, io::Write};

    use judger::{lang::gnu_cpp17_o2, Error, OneOff, Status};

    const A_PLUS_B_RAW: &str = r#"
#include <iostream>

using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
"#;

    #[test]
    fn test_gnu_cpp() -> Result<(), Error> {
        let dir = tempfile::tempdir().unwrap();
        
        let src = dir.path().join("main.cpp");
        let mut fsrc = File::create(&src).unwrap();
        write!(fsrc, "{}", A_PLUS_B_RAW).unwrap();

        let mut one = OneOff::new(src.into(), gnu_cpp17_o2());
        one.set_wd(dir.path().to_path_buf());

        eprintln!("cwd: {}", std::env::current_dir().unwrap().display());
        let res = one.exec()?;
        if let Status::Accepted = res.status {
            eprintln!("res = {:?}", res);
        } else {
            panic!("compile failed")
        }
        Ok(())
    }
}
