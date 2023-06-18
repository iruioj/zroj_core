use std::io;

use crate::{
    data::{tempdir_unzip, Data, StoreFile},
    DataError, Override, RuntimeError,
};

use store::{FsStore, Handle};
use tempfile::TempDir;

/// 题目的存储
pub struct ProblemStore<T, M, S>
where
    T: FsStore,
    M: FsStore,
    S: Override<M> + FsStore,
{
    /// 临时文件夹
    #[allow(unused)]
    dir: TempDir,
    data: Data<T, M, S>,
}

impl<T, M, S> ProblemStore<T, M, S>
where
    T: FsStore,
    M: FsStore,
    S: Override<M> + FsStore,
{
    /// 从 reader 中解压 zip 文件到一个临时文件夹中，然后解析为题目数据
    pub fn unzip_reader(reader: impl io::Read + io::Seek) -> Result<Self, DataError> {
        let dir = tempdir_unzip(reader)?;
        let ctx = store::Handle::new(dir.path());
        Ok(Self {
            dir,
            data: FsStore::open(ctx)?,
        })
    }
    /// get read only data
    pub fn data(&self) -> &Data<T, M, S> {
        &self.data
    }
}

pub trait JudgeProblem {
    type T: FsStore;
    type M: FsStore;
    type S: FsStore + Override<Self::M>;
    type Subm: FsStore;

    /// 单个测试点的评测
    ///
    /// 注意，源文件的编译、checker 的编译等等事情也会放在这里一起做。
    /// 从“多测试点评测”的概念上看，其最本质的写法就是对不同的测试点，把所有的流程都走一遍。
    /// 当然我们可以在实现的时候结合缓存系统来提高效率。
    fn judge_task(
        &self,
        judger: impl judger::Judger,
        meta: &mut Self::M,
        task: &mut Self::T,
        subm: &mut Self::Subm,
    ) -> Result<judger::TaskReport, RuntimeError>;
}

/// 自动编译文件，可执行文件名为 name，编译日志为 name.c.log
fn compile_in_wd(
    file: &mut StoreFile,
    wd: &Handle,
    name: impl AsRef<str>,
) -> Result<judger::sandbox::Termination, RuntimeError> {
    use judger::Compile;
    let src = wd.join(String::from(name.as_ref()) + file.file_type.ext());
    let exec = wd.join(name.as_ref());
    let clog = wd.join(String::from(name.as_ref()) + ".c.log");

    file.copy_all(&mut src.create_new_file().map_err(DataError::from)?).unwrap();

    let term = file
        .file_type
        .compile(&src, &exec, &clog)
        .exec_fork()
        .unwrap();
    Ok(term)
}
fn copy_in_wd(
    file: &mut StoreFile,
    wd: &Handle,
    name: impl AsRef<str>,
) -> Result<(), DataError> {
    let src = wd.join(name.as_ref());
    file.copy_all(&mut src.create_new_file()?).unwrap();
    Ok(())
}

pub mod traditional;
