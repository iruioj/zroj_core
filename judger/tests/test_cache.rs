#[cfg(test)]

mod test_cache {
    use std::{fs::File, io::Write, path::PathBuf, time::Instant};

    use judger::{cache::Cache, lang::Builtin};

	use sandbox::{Status as Stat};

	use regex::Regex;

	fn write_file(dir: PathBuf, name: &str, content: &str) -> PathBuf {
		let file_path = dir.as_path().join(name);
		let mut file = File::create(&file_path).unwrap();
		write!(file, "{}", content).unwrap();
		file_path
	}
	
	#[test]
	fn test_cache() {
        let dir = tempfile::tempdir().unwrap().into_path();

		let ok = write_file(dir.clone(), "ok.cpp", r#"
#include <bits/stdc++.h>
using namespace std;

int main() {
	int a, b;
	cin >> a >> b;
	cout << a + b << endl;
	return 0;
}
		"#);

		let ce = write_file(dir.clone(), "ce.cpp", r#"
#include <bits/stdc++.h>

int main() {
	int a, b;
	cin >> a >> b;
	cout << a + b << endl;
	return 0;
}
		"#);

		let mut cache = Cache::new(3u64, dir);

		let regex = Regex::new("^*316eb79746f1ea7da6e1165db3936335ea0af7005267be4020b782040ff60e57$").unwrap();

		let now = Instant::now();
		let mut v = vec![];

		for _ in 0..3 {
			let Ok(ok_exec) = cache.get_exec(&Builtin::GnuCpp17O2, &ok) else { panic!(); };
			let Err(ce_info) = cache.get_exec(&Builtin::GnuCpp17O2, &ce) else { panic!(); };

			let Stat::RuntimeError(x, s) = ce_info else { panic!(); };
			
			assert!(regex.is_match(&format!("{}", ok_exec.display())));
			assert_eq!(x, 1);
			assert_eq!(s, None);
			
			v.push(now.elapsed().as_nanos());
		}

		assert!(v[2] < 2 * v[0]);
	}
}