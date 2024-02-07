use std::collections::BTreeMap;

use store::{FsStore, Handle};

#[derive(FsStore, PartialEq)]
struct MyU64 {
    #[meta]
    inner: u64,
}

#[test]
fn test_btree_map() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut a: BTreeMap<u64, MyU64> = [
        (1, MyU64 { inner: 2 }),
        (3, MyU64 { inner: 4 }),
        (5, MyU64 { inner: 6 }),
    ]
    .into_iter()
    .collect();
    let ctx = Handle::new(dir.path());
    FsStore::save(&mut a, &ctx).unwrap();
    dbg!(&ctx);
    let a2: BTreeMap<u64, MyU64> = FsStore::open(&ctx).unwrap();
    assert!(a == a2);
    drop(dir);
}
