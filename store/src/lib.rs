mod error;

pub use error::Error;
use serde::{de::DeserializeOwned, Serialize};

use std::{
    fmt::Debug,
    io::Seek,
    path::{Path, PathBuf},
};

// Re-export #[derive(FsStore)]
#[allow(unused_imports)]
#[macro_use]
extern crate store_derive;
pub use store_derive::FsStore;

/// 文件系统中的一个文件或文件夹，不保证其存在性
///
#[derive(Clone)]
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
    pub fn open_file(&self) -> Result<std::fs::File, Error> {
        if self.dir.is_file() {
            std::fs::File::open(self).map_err(Error::OpenFile)
        } else {
            Err(Error::NotFile)
        }
    }
    /// 在该路径下新建文件，会自动补齐父级目录，要求其不存在
    pub fn create_new_file(&self) -> Result<std::fs::File, Error> {
        if let Some(par) = self.dir.parent() {
            std::fs::create_dir_all(par).map_err(Error::CreateParentDir)?;
        }
        std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(self.as_ref())
            .map_err(Error::CreateNewFile)
    }
    /// 从该路径下的文件中解析数据（要求文件存在）
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_reader(self.open_file()?).map_err(Error::Deserialize)
    }
    /// 将数据序列化到该路径下（要求文件不存在）
    pub fn serialize_new_file<T: Serialize>(&self, data: &T) -> Result<(), Error> {
        serde_json::to_writer(self.create_new_file()?, data).map_err(Error::Serialize)
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
        if prefix.is_none() {
            writeln!(f, "{}", slug.as_ref().display())?;
        } else if is_last {
            writeln!(
                f,
                "{}└── {}",
                prefix.clone().unwrap(),
                slug.as_ref().display()
            )?;
        } else {
            writeln!(
                f,
                "{}├── {}",
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

/// 文件夹数据结构化存储
pub trait FsStore: Sized {
    /// 读取文件（夹）下的数据信息
    fn open(ctx: Handle) -> Result<Self, Error>;

    /// 将该结构体中的信息保存到一个文件（夹）中
    fn save(&mut self, ctx: Handle) -> Result<(), Error>;
}

impl FsStore for () {
    fn open(_: Handle) -> Result<Self, Error> {
        Ok(())
    }

    fn save(&mut self, _: Handle) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! impl_tuple {
    ( $( $type:ident ),*  ) => {

impl< $( $type : FsStore, )* > FsStore for ( $( $type, )* ) {
    fn open(ctx: Handle) -> Result<Self, Error> {
        Ok((
            $( FsStore::open(ctx.join(stringify!($type)))?, )*
        ))
    }

    fn save(&mut self, ctx: Handle) -> Result<(), Error> {
        #[allow(non_snake_case)]
        let ( $( $type, )* ) = self;
        $( FsStore::save($type, ctx.join(stringify!($type)))?; )*
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
const VEC_FS_STORE_LIMIT: usize = 512;

impl<T: FsStore> FsStore for Vec<T> {
    fn open(ctx: Handle) -> Result<Self, Error> {
        let len = ctx.join("_vec_len").deserialize::<usize>()?;
        (0..len)
            .map(|i| T::open(ctx.join(format!("item_{i}"))))
            .collect()
    }

    fn save(&mut self, ctx: Handle) -> Result<(), Error> {
        if self.len() > VEC_FS_STORE_LIMIT {
            return Err(Error::VecTooLong);
        }
        ctx.join("_vec_len").serialize_new_file(&self.len())?;
        for (i, item) in self.iter_mut().enumerate() {
            item.save(ctx.join(format!("item_{i}")))?;
        }
        Ok(())
    }
}

impl FsStore for std::fs::File {
    fn open(ctx: Handle) -> Result<Self, Error> {
        ctx.open_file()
    }

    fn save(&mut self, ctx: Handle) -> Result<(), Error> {
        let mut dest = ctx.create_new_file()?;
        self.seek(std::io::SeekFrom::Start(0)).unwrap();
        std::io::copy(self, &mut dest).unwrap();
        Ok(())
    }
}

#[macro_use]
#[allow(unused_imports)]
extern crate serde;
pub use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
