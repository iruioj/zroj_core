# zroj_core/store

This package implements the serialization and deserialization of structural data over naive file system,
acting as an enhancement of `serde`, which lacks the ability of handling large files.

To store a struct into a file/directory, one can derive the `FsStore` trait:

For a struct to auto derive [`FsStore`], each of its fields must either

- derives [`serde::Serialize`] and [`serde::Deserialize`], and marked with `#[meta]` attribute.
- derives [`FsStore`].

Deriviation of `sdt::fs::File` for `FsStore` has been implemented.

To save/load a struct from a file/directory, one resolves to the [`Handle`]:

```rust
# use store::FsStore;
# use store::Handle;
# use std::io::Write;
/// A file with custom kind
#[derive(FsStore, Debug)]
pub struct StoreFile {
    pub file: std::fs::File,
    #[meta]
    pub kind: String,
}

# let mut file = tempfile::tempfile().unwrap();
# file.write_all("hello!".as_bytes()).unwrap();
let mut store_file = StoreFile {
    file,
    kind: "text".into()
};

# let temp_dir = tempfile::tempdir().unwrap();
let ctx = Handle::new(temp_dir.path());
FsStore::save(&mut store_file, &ctx).unwrap();
dbg!(&ctx);
let _: StoreFile = FsStore::open(&ctx).unwrap();
# drop(temp_dir);
```