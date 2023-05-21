use std::io;

use crate::{
    data::{tempdir_unzip, Data},
    Error, Override,
};

use store::FsStore;
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
    pub fn unzip_reader(reader: impl io::Read + io::Seek) -> Result<Self, Error> {
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
        meta: &Self::M,
        task: &Self::T,
        subm: Self::Subm,
    ) -> Result<judger::TaskReport, Error>;
}

pub mod traditional {
    use super::JudgeProblem;
    use crate::data::StoreFile;
    use judger::Compile;
    use store::FsStore;

    #[derive(FsStore)]
    pub struct Meta {
        pub checker: StoreFile,
        // pub validator: String,
        /// 时间限制
        #[meta]
        pub time_limit: u32,
        /// 空间限制
        #[meta]
        pub memory_limit: u32,
    }

    #[derive(FsStore)]
    pub struct Task {
        pub input: StoreFile,
        pub output: StoreFile,
    }

    #[derive(FsStore)]
    pub struct Subm {
        source: StoreFile,
    }

    /// 传统题（只是一个评测，数据直接用 ProblemStore 存）
    pub struct Traditional;

    impl JudgeProblem for Traditional {
        type T = Task;
        type M = Meta;
        type S = ();
        type Subm = Subm;

        // 先写了一个粗糙的，后面再来错误处理
        fn judge_task(
            &self,
            judger: impl judger::Judger,
            _meta: &Self::M,
            _task: &Self::T,
            subm: Self::Subm,
        ) -> Result<judger::TaskReport, crate::Error> {
            let wd = judger.working_dir();
            let Subm { mut source } = subm;

            let src = wd.join(String::from("source") + source.file_type.ext());
            let exec = wd.join("main");

            source.copy_all(&mut src.open_file()?).unwrap();

            let compile_cmd = source.file_type.compile(&src, &exec);

            let term = compile_cmd.exec_fork().unwrap();

            // Compile Error
            if !term.status.ok() {
                return Ok(judger::TaskReport {
                    status: judger::Status::CompileError(term.status),
                    time: term.cpu_time,
                    memory: term.memory,
                    // todo: add log
                    payload: vec![],
                });
            }

            todo!()
        }
    }
}
