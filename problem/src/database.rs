use std::{
    collections::HashSet,
    fs,
    io::{Read, Seek},
    marker::PhantomData,
    path::PathBuf,
};

use store::{FsStore, Handle};

pub type ProbID = u32;

#[derive(Debug)]
pub enum ProbSetError {
    /// 插入题目时 id 重复
    DuplicateID(u32),
    ReadZip(zip::result::ZipError),
    OpenStore(store::Error),
    InvalidID(u32),
    Remove(std::io::Error),
}

impl std::fmt::Display for ProbSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbSetError::DuplicateID(id) => write!(f, "duplicate id {id}"),
            ProbSetError::ReadZip(e) => write!(f, "reading zip file: {e}"),
            ProbSetError::OpenStore(e) => write!(f, "open store: {e}"),
            ProbSetError::InvalidID(id) => write!(f, "invalid id {id}"),
            ProbSetError::Remove(e) => write!(f, "remove files: {e}"),
        }
    }
}

impl std::error::Error for ProbSetError {}

/// 存储类型为 P 的有关题目的数据，以题目 ID 作为键值
/// 
/// 由于修改操作远少于读取操作，因此就不考虑多线程的优化了，直接外面套一个 Lock
pub struct DataSet<P: FsStore> {
    path: Handle,
    indices: HashSet<ProbID>,
    _marker: PhantomData<P>,
}

impl<P: FsStore> DataSet<P> {
    pub fn read_dir(path: PathBuf) -> Result<Self, std::io::Error> {
        let indices = fs::read_dir(&path)?
            .map_while(Result::ok)
            .map_while(|dir| dir.file_name().to_str().map(|s| s.parse::<ProbID>()))
            .map_while(Result::ok)
            .collect();
        Ok(Self {
            path: Handle::new(path),
            indices,
            _marker: PhantomData,
        })
    }
    /// get a problem from id
    pub fn get(&self, id: ProbID) -> Result<P, ProbSetError> {
        P::open(&self.path.join(id.to_string())).map_err(ProbSetError::OpenStore)
    }
    /// remove a problem by id
    pub fn remove(&mut self, id: ProbID) -> Result<(), ProbSetError> {
        if self.indices.contains(&id) {
            fs::remove_dir_all(self.path.join(id.to_string())).map_err(ProbSetError::Remove)
        } else {
            Err(ProbSetError::InvalidID(id))
        }
    }
    /// insert a problem from zip data
    ///
    /// 使用 get 来验证数据格式的合法性
    pub fn insert_zip(&mut self, id: ProbID, reader: impl Read + Seek) -> Result<P, ProbSetError> {
        if self.indices.contains(&id) {
            return Err(ProbSetError::DuplicateID(id));
        }
        let mut zip = zip::read::ZipArchive::new(reader).map_err(ProbSetError::ReadZip)?;
        zip.extract(self.path.join(id.to_string()))
            .map_err(ProbSetError::ReadZip)?;
        self.get(id)
    }
}
