use std::io::{Seek, Write};

use store::{FsStore, Handle};

#[derive(store_derive::FsStore)]
struct TestStore {
    #[meta]
    flag: bool,
    #[meta]
    num: isize,
    file: std::fs::File,
}

#[test]
fn test_derive_fs_store() {
    let mut file = tempfile::tempfile().unwrap();
    file.write_all("hello!".as_bytes()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();

    let mut store = TestStore {
        flag: false,
        num: 23,
        file,
    };
    let dir = tempfile::tempdir().unwrap();
    // let handle = Handle::new(".");
    let handle = Handle::new(dir.path());
    store.save(handle.join("test_store")).unwrap();
    let mut store2 = TestStore::open(handle.join("test_store")).unwrap();
    store2.file.seek(std::io::SeekFrom::Start(0)).unwrap();
    let content = std::io::read_to_string(&mut store2.file).unwrap();
    assert_eq!(content, "hello!");
}
