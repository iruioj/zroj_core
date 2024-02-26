#![doc = include_str!("../README.md")]

mod error;

pub type Error = error::StoreError;
use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fmt::Debug,
    fs,
    io::{Read, Seek, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    str::FromStr,
};

/// 文件系统中的一个文件或文件夹的句柄，不保证其存在性
#[derive(Clone, Serialize, Deserialize)]
pub struct Handle {
    dir: PathBuf,
}

impl Handle {
    /// create a new handle from os path
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            dir: path.as_ref().to_path_buf(),
        }
    }
    /// 在末尾添加一个新的文件夹/路径
    pub fn join(&self, p: impl AsRef<Path>) -> Self {
        Self {
            dir: self.dir.join(p),
        }
    }
    /// 打开该路径下的文件（要求其必须存在）
    pub fn open_file(&self) -> Result<fs::File, Error> {
        if self.dir.is_file() {
            Ok(fs::File::open(self).context("open file")?)
        } else {
            Err(anyhow::anyhow!(
                "try to open a non-file item: {:?}",
                self.path()
            ))?
        }
    }
    /// 在该路径下新建文件用于写入，会自动补齐父级目录，要求其不存在
    pub fn create_new_file(&self) -> Result<fs::File, Error> {
        if let Some(par) = self.dir.parent() {
            fs::create_dir_all(par).context("create parent dir")?;
        }
        Ok(fs::File::options()
            .write(true)
            .create_new(true)
            .open(self.as_ref())
            .with_context(|| format!("create new file {:?}", self.dir))?)
    }
    /// 如果是文件就删除，如果是文件夹就删除它自己和所有子文件夹和子文件
    pub fn remove_all(&self) -> Result<(), Error> {
        if !self.path().exists() {
            return Ok(());
        }
        if self.path().is_dir() {
            Ok(fs::remove_dir_all(self.path()).context("remove dir all")?)
        } else {
            Ok(fs::remove_file(self.path()).context("remove file")?)
        }
    }
    /// remove original item (recursively) and create an empty directory
    pub fn prepare_empty_dir(&self) -> Result<(), Error> {
        self.remove_all()?;
        std::fs::create_dir_all(self.path()).context("create dir")?;
        Ok(())
    }
    /// 从该路径下的文件中解析数据（要求文件存在）
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, Error> {
        Ok(serde_json::from_reader(self.open_file()?).context("deserialize json")?)
    }
    /// 将数据序列化到该路径下（要求文件不存在）
    pub fn serialize_new_file<T: Serialize>(&self, data: &T) -> Result<(), Error> {
        Ok(serde_json::to_writer(self.create_new_file()?, data).context("serialize json")?)
    }
    pub fn serialize_pretty_new_file<T: Serialize>(&self, data: &T) -> Result<(), Error> {
        Ok(serde_json::to_writer_pretty(self.create_new_file()?, data).context("serialize json")?)
    }
    pub fn path(&self) -> &Path {
        &self.dir
    }
    /// See [`Path::with_extension`]
    pub fn with_extension<S: AsRef<std::ffi::OsStr>>(&self, ext: S) -> Self {
        Self {
            dir: self.dir.with_extension(ext),
        }
    }
    pub fn to_cstring(&self) -> CString {
        CString::new(self.path().as_os_str().as_bytes()).unwrap()
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.path().to_str().expect("handle to str failed");
        f.write_str(str)
    }
}

impl Handle {
    fn _fmt(
        &self,
        prefix: Option<String>,
        slug: impl AsRef<Path>,
        is_last: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let kind = if self.dir.is_dir() {
            "dir"
        } else if self.dir.is_file() {
            "file"
        } else if !self.dir.exists() {
            "non-exist"
        } else {
            "unknown"
        };

        if prefix.is_none() {
            writeln!(f, "{} [{kind}]", slug.as_ref().display())?;
        } else if is_last {
            writeln!(
                f,
                "{}└── {} [{kind}]",
                prefix.clone().unwrap(),
                slug.as_ref().display()
            )?;
        } else {
            writeln!(
                f,
                "{}├── {} [{kind}]",
                prefix.clone().unwrap(),
                slug.as_ref().display()
            )?;
        }
        let prefix = prefix
            .map(|p| p + if is_last { "    " } else { "│   " })
            .unwrap_or_default();
        if self.dir.is_dir() {
            let mut items = self
                .dir
                .read_dir()
                .expect("read dir error")
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            if items.is_empty() {
                return Ok(());
            }
            items.sort_by_cached_key(|d| d.file_name());
            let (last, others) = items.split_last_mut().unwrap();
            for dir in others {
                let slug = dir.file_name();
                self.join(&slug)
                    ._fmt(Some(prefix.clone()), slug, false, f)?;
            }
            let slug = last.file_name();
            self.join(&slug)._fmt(Some(prefix), slug, true, f)?;
        }
        Ok(())
    }
}
impl Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._fmt(None, self.dir.to_str().unwrap(), true, f)
    }
}

impl AsRef<Path> for Handle {
    fn as_ref(&self) -> &Path {
        self.dir.as_ref()
    }
}

/// Copy files from source to destination recursively.
pub fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// 文件夹数据结构化存储
pub trait FsStore: Sized {
    /// 读取文件（夹）下的数据信息
    fn open(ctx: &Handle) -> Result<Self, Error>;

    /// 将该结构体中的信息保存到一个文件（夹）中
    fn save(&mut self, ctx: &Handle) -> Result<(), Error>;

    fn clone_to(&mut self, ctx: &Handle) -> Result<Self, Error> {
        self.save(ctx)?;
        FsStore::open(ctx)
    }

    /// Save data to the destination and reopen it at new location.
    fn save_as(&mut self, ctx: &Handle) -> Result<(), Error> {
        *self = self.clone_to(ctx)?;
        Ok(())
    }

    /// first save to a temporary directory, then rename it to the destination
    fn safe_save(&mut self, ctx: &Handle) -> Result<(), Error> {
        let dir = tempfile::tempdir().context("create tmp dir")?;
        let tmpdest = Handle::new(dir.path().join("dest"));
        self.save(&tmpdest)
            .with_context(|| format!("save to tmp dir {}", tmpdest.path().display()))?;
        ctx.remove_all().context("remove original dest data")?;
        if let Some(par) = ctx.path().parent() {
            if !par.exists() {
                fs::create_dir_all(par).context("create parent dir before renaming")?;
            }
        }
        // try to rename; if failed, then do copy
        fs::rename(tmpdest.path(), ctx.path())
            .context("rename dir during safe_save")
            .or_else(|_| {
                if tmpdest.path().is_dir() {
                    copy_recursively(tmpdest.path(), ctx.path())
                        .context("copy dir recursively during safe_save")
                } else {
                    std::fs::copy(tmpdest.path(), ctx.path())
                        .map(|_| ())
                        .context("copy file during save_safe")
                }
            })
            .context("rename/copy tmp data to dest")?;
        Ok(())
    }
}

impl FsStore for () {
    fn open(_: &Handle) -> Result<Self, Error> {
        Ok(())
    }

    fn save(&mut self, _: &Handle) -> Result<(), Error> {
        Ok(())
    }
}

impl FsStore for String {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        Ok(std::fs::read_to_string(ctx).context("read to string")?)
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        Ok(ctx
            .create_new_file()?
            .write_all(self.as_bytes())
            .context("write string to file")?)
    }
}

/// a helper struct for store serializable data in file
pub struct SerdeFsStore<T: serde::Serialize + for<'a> serde::Deserialize<'a>>(pub T);

impl<T: serde::Serialize + for<'a> serde::Deserialize<'a>> FsStore for SerdeFsStore<T> {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        Ok(Self(ctx.deserialize()?))
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        ctx.serialize_new_file(&self.0)
    }
}

macro_rules! impl_tuple {
    ( $( $type:ident ),*  ) => {

impl< $( $type : FsStore, )* > FsStore for ( $( $type, )* ) {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        Ok((
            $( FsStore::open(&ctx.join(stringify!($type)))?, )*
        ))
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        #[allow(non_snake_case)]
        let ( $( $type, )* ) = self;
        $( FsStore::save($type, &ctx.join(stringify!($type)))?; )*
        Ok(())
    }
}

    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);

/// 自动序列化时 vector 中元素数量上限，防止产生过多的文件
const VEC_FS_STORE_LIMIT: usize = 2048;

impl<T: FsStore> FsStore for Vec<T> {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        let mut i = 0;
        let mut r = Self::default();
        loop {
            let path = ctx.join(format!("v{i}"));
            if path.path().exists() {
                r.push(T::open(&path)?);
            } else {
                break;
            }
            i += 1;
        }
        Ok(r)
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        if self.len() > VEC_FS_STORE_LIMIT {
            return Err(anyhow::anyhow!("vec too long"))?;
        }
        for (i, item) in self.iter_mut().enumerate() {
            item.save(&ctx.join(format!("v{i}")))?;
        }
        Ok(())
    }
}

impl FsStore for Vec<u8> {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        let mut file = ctx.open_file()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).context("read file to buffer")?;
        Ok(buf)
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        let mut file = ctx.create_new_file()?;
        file.write_all(&self).context("write buffer to file")?;
        Ok(())
    }
}

impl FsStore for fs::File {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        ctx.open_file()
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        let mut dest = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(
                self.metadata()
                    .context("get original file permission")?
                    .permissions()
                    .mode(),
            )
            .open(ctx)
            .with_context(|| format!("save fs::File to {:?}", ctx.path()))?;
        self.seek(std::io::SeekFrom::Start(0)).unwrap();
        std::io::copy(self, &mut dest).context("copying data")?;
        Ok(())
    }
}

impl FsStore for Option<fs::File> {
    fn open(ctx: &Handle) -> Result<Self, Error> {
        if ctx.path().exists() {
            Ok(Some(ctx.open_file()?))
        } else {
            Ok(None)
        }
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        if let Some(f) = self {
            f.save(ctx)?;
        }
        Ok(())
    }
}

impl<T: Send> FsStore for std::marker::PhantomData<T> {
    fn open(_: &Handle) -> Result<Self, Error> {
        Ok(Self)
    }

    fn save(&mut self, _: &Handle) -> Result<(), Error> {
        Ok(())
    }
}

impl<K, V> FsStore for BTreeMap<K, V>
where
    V: FsStore,
    K: ToString + FromStr + Ord,
{
    fn open(ctx: &Handle) -> Result<Self, Error> {
        ctx.dir
            .read_dir()
            .context("read dir")?
            .filter_map(|e| {
                let e = e.ok()?;
                let binding = e.file_name();
                let key = binding.to_str()?;
                let k = K::from_str(key).ok()?; // ignore invalid key

                Some(V::open(&ctx.join(key)).map(|v| (k, v)))
            })
            .collect()
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        self.iter_mut()
            .try_fold((), |_, (k, v)| v.save(&ctx.join(k.to_string())))
    }
}

impl<V> FsStore for BTreeSet<V>
where
    V: ToString + FromStr + Ord,
    <V as FromStr>::Err: Debug,
{
    fn open(ctx: &Handle) -> Result<Self, Error> {
        let data: BTreeSet<String> = ctx.deserialize()?;
        Ok(data
            .into_iter()
            .map(|s| FromStr::from_str(&s).expect("invalid string"))
            .collect::<BTreeSet<V>>())
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), Error> {
        let data: BTreeSet<String> = self.iter().map(|x| x.to_string()).collect();
        ctx.serialize_new_file(&data)
    }
}

#[macro_use]
#[allow(unused_imports)]
extern crate serde;
pub use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

// Re-export #[derive(FsStore)]
#[allow(unused_imports)]
#[macro_use]
extern crate store_derive;
pub use store_derive::FsStore;
