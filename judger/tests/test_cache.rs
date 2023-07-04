/*
#[cfg(test)]

mod test_cache {
    use std::{fs::File, io::Write, path::PathBuf, time::Instant};

    use judger::{cache::Cache, FileType, StoreFile};

    use sandbox::Status as Stat;

	use store::Handle;

    fn write_file(dir: PathBuf, name: &str, content: &str) -> PathBuf {
        let file_path = dir.as_path().join(name);
        let mut file = File::create(&file_path).unwrap();
        write!(file, "{}", content).unwrap();
        file_path
    }

    #[test]
    fn test_cache() {
        let dir = tempfile::tempdir().unwrap().into_path();

        let ok = write_file(
            dir.clone(),
            "ok.cpp",
            r#"
#include <iostream>
using namespace std;

int main() {
	int a, b;
	cin >> a >> b;
	cout << a + b << endl;
	return 0;
}
		"#,
        );

        let ce = write_file(
            dir.clone(),
            "ce.cpp",
            r#"
#include <iostream>

int main() {
	int a, b;
	cin >> a >> b;
	cout << a + b << endl;
	return 0;
}
		"#,
        );

        let mut cache = Cache::new(3, Handle::new(dir));

        let now = Instant::now();
        let mut v = vec![];

        for _ in 0..3 {
			let mut ok_file = StoreFile {
				file: File::create(ok.clone()).unwrap(),
				file_type: FileType::GnuCpp17O2,
			};
			let mut ce_file = StoreFile {
				file: File::create(ce.clone()).unwrap(),
				file_type: FileType::GnuCpp17O2,
			};
            let Ok((ok_exec, ok_clog)) = cache.get_exec(&mut ok_file) else { panic!(); };
            let Err(ce_info) = cache.get_exec(&mut ce_file) else { panic!(); };

            let judger::Error::CacheCE((Stat::RuntimeError(x, s), clog)) = ce_info else { panic!(); };

            assert_eq!(x, 1);
            assert_eq!(s, None);

            v.push(now.elapsed().as_nanos());
        }

        assert!(v[2] < 2 * v[0]);
    }
}
*/