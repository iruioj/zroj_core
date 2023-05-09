use crate::error::Result;
use std::{fs, io, path::PathBuf};

/// 一个只读的可评测的题目的数据
///
/// 其本体保存于文件系统中的某个（临时）文件
///
/// 不会涉及到对数据文件的修改。
pub trait Problem
where
    Self: Sized,
{
    /// 题目的元数据。
    ///
    /// 对于内置题目，元数据以 enum Builtin 的形式给出，这样只需要实现一个 Problem<Meta = BuiltinMeta>
    /// 即可。而如果想要定义新的题目类型，可以自己实现对应的 ProblemSet。这不无道理，
    /// 因为新形式的题目很可能与老题目的评测方式完全不同，可以作为一个新的题目集。
    /// 而如果要在一个题目集中加入新的题目类型，那就得在 Builtin 里加东西，
    /// 那么建议自己修改源代码和前后端然后编译。
    type Meta: Sized;

    /// 读取题目时返回的文件类型
    type File: io::Read + io::Seek;

    /// 从一个文件读取题目
    fn open(path: &PathBuf) -> Result<Self>;

    /// 该题目的文件路径
    fn path(&self) -> &PathBuf;

    fn meta(&self) -> &Self::Meta;

    /// 读取题目数据中的文件
    fn open_file(&self, path: &PathBuf) -> Result<Self::File>;

    /// 不会验证 reader 里的内容是否合法
    fn replace_reader(&self, reader: &mut (impl io::Read + io::Seek)) -> Result<()> {
        // create a file if it does not exist, and will truncate it if it does.
        let mut file = fs::File::create(self.path())?;
        io::copy(reader, &mut file)?;
        Ok(())
    }

    /// 不会验证 file 里的内容是否合法
    fn replace(&self, file: &PathBuf) -> Result<()> {
        let mut this = fs::File::create(self.path())?;
        let mut file = fs::File::open(file)?;
        io::copy(&mut file, &mut this)?;
        Ok(())
    }
}

pub mod zip {
    use std::fs;
    use std::path::PathBuf;

    use crate::error::Result;
    use crate::{Builtin, Error::*};

    /// 在打开压缩文件时解压到临时文件夹
    pub struct ZipProblem {
        path: PathBuf,
        dir: tempfile::TempDir,
        meta: Builtin,
    }

    impl super::Problem for ZipProblem {
        type Meta = Builtin;
        type File = fs::File;

        fn open(path: &PathBuf) -> Result<Self> {
            let dir = tempfile::tempdir()?;
            let file = fs::File::open(path)?;
            let mut zip = zip::ZipArchive::new(file)?;

            // extract to temporary dir
            // not atomic
            zip.extract(dir.path())?;

            let meta = {
                let meta = zip.by_name("meta")?;
                // let meta = std::io::read_to_string(meta)?;
                let meta: Builtin = serde_json::from_reader(meta)?;
                meta
            };
            // 获取 comment 的第一个字节
            Ok(Self {
                path: path.clone(),
                dir,
                meta,
            })
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }

        fn meta(&self) -> &Self::Meta {
            &self.meta
        }

        fn open_file(&self, relative_path: &PathBuf) -> Result<Self::File> {
            Ok(fs::File::open(self.dir.path().join(relative_path))?)
        }
    }
}
