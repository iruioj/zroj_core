use judger::{SourceFile, StoreFile};
use server::manager::OneOffManager;

const SRC: &str = r"
#include<iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
";

fn main() {
    let dir = tempfile::TempDir::new().unwrap();
    let oneoff = OneOffManager::new(dir.path()).unwrap();

    let source = SourceFile::from_str(SRC, judger::FileType::GnuCpp17O2);
    let input = StoreFile::from_str(r"1 2", judger::FileType::Plain);

    let h = std::thread::spawn(move || {
        oneoff.add_test(0, source, input).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        let r = oneoff.get_result(&0).unwrap().unwrap();
        assert!(r.meta.status == judger::Status::Good);
        dbg!(r);
    });
    h.join().unwrap();

    drop(dir)
}
