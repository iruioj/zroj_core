#[cfg(test)]

mod test_cache {
    use std::{fs::File, io::Write, path::PathBuf, time::Instant};

    use judger::{cache::Cache, FileType};

    use sandbox::Status as Stat;

    use regex::Regex;

    fn write_file(dir: PathBuf, name: &str, content: &str) -> PathBuf {
        let file_path = dir.as_path().join(name);
        let mut file = File::create(&file_path).unwrap();
        write!(file, "{}", content).unwrap();
        file_path
    }

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

        let mut cache = Cache::new(3u64, dir);

        let regex =
            Regex::new("^*67333e3cb9085dbb0fd301fd4809a98cc8af31966b515e9340720367dd4380bf$")
                .unwrap();

        let now = Instant::now();
        let mut v = vec![];

        for _ in 0..3 {
            let Ok(ok_exec) = cache.get_exec(&FileType::GnuCpp17O2, &ok) else {
                panic!();
            };
            let Err(ce_info) = cache.get_exec(&FileType::GnuCpp17O2, &ce) else {
                panic!();
            };

            let judger::Error::CacheCE(Stat::RuntimeError(x)) = ce_info else {
                panic!();
            };

            dbg!(ok_exec.display());
            assert!(regex.is_match(&format!("{}", ok_exec.display())));
            assert_eq!(x, 1);

            v.push(now.elapsed().as_nanos());
        }

        assert!(v[2] < 2 * v[0]);
    }
}
