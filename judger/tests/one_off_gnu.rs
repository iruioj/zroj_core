#[cfg(test)]
mod one_off {
    use std::{fs::File, io::Write};

    use judger::{lang::gnu_cpp17_o2, Error, OneOff, Status};

    #[test]
    fn test_gnu_cpp() -> Result<(), Error> {
        let a_plus_b_raw = r#"
#include <iostream>

using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
"#;
        let input_content = "1 2";

        let dir = tempfile::tempdir().unwrap();

        let src = dir.path().join("main.cpp");
        let mut fsrc = File::create(&src).unwrap();
        write!(fsrc, "{}", a_plus_b_raw).unwrap();

        let inp = dir.path().join("input.txt");
        let mut finp = File::create(&inp).unwrap();
        write!(finp, "{}", input_content).unwrap();

        let mut one = OneOff::new(src.into(), inp.into(), gnu_cpp17_o2());
        one.set_wd(dir.path().to_path_buf());

        let res = one.exec()?;
        if let Status::Accepted = res.status {
            eprintln!("res = {:?}", res);
            assert_eq!(res.stdout.0, "3\n");
        } else {
            panic!("compile failed")
        }
        Ok(())
    }
}