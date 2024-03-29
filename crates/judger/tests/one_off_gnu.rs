#[cfg(test)]
mod one_off {
    use judger::{FileType, OneOff, SourceFile, Status, StoreFile};
    use store::Handle;

    #[test]
    fn test_gnu_cpp() -> anyhow::Result<()> {
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

        let src = SourceFile::from_str(a_plus_b_raw, FileType::GnuCpp17O2);
        let inp = StoreFile::from_str(input_content, FileType::Plain);

        let mut one = OneOff::new(src, inp);
        one.set_wd(Handle::new(&dir));

        let res = one.exec()?;
        if let Status::Good = res.meta.status {
            eprintln!("res = {:#?}", res);
            assert_eq!(String::from(&res.payload[0].1), "3\n");
        } else {
            panic!("not accepted, res = {:?}", res)
        }
        drop(dir);
        Ok(())
    }
}
