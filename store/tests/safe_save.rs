use std::collections::BTreeMap;

use store::{FsStore, Handle};

#[derive(FsStore)]
struct Str {
    #[meta]
    inner: String,
}

impl From<&str> for Str {
    fn from(value: &str) -> Self {
        Self {
            inner: value.to_string(),
        }
    }
}

#[test]
fn test_safe_save() {
    let mut data: BTreeMap<String, Str> = [
        ("a".into(), "b".into()),
        ("c".into(), "d".into()),
        ("e".into(), "f".into()),
    ]
    .into();

    let dir = tempfile::tempdir().unwrap();
    let ctx = Handle::new(dir.path());
    FsStore::safe_save(&mut data, &ctx).unwrap();

    dbg!(ctx);
    drop(dir);
}
