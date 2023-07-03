#[cfg(test)]
mod one_off {
    use judger::{Error, FileType, OneOff, Status, StoreFile};
    use store::Handle;

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

        let mut src = StoreFile::create_tmp(a_plus_b_raw);
        src.file_type = FileType::GnuCpp17O2;

        let inp = StoreFile::create_tmp(input_content);

        let mut one = OneOff::new(src, Some(inp));
        one.set_wd(Handle::new(&dir));

        let res = one.exec()?;
        if let Status::Accepted = res.status {
            eprintln!("res = {:?}", res);
            assert_eq!(String::from(&res.payload[1].1), "3\n");
        } else {
            panic!("not accepted, res = {:?}", res)
        }
        drop(dir);
        Ok(())
    }
}
