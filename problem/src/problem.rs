use crate::{data::Data, Error};

use store::FsStore;

/// 不会涉及到对数据文件的修改。
pub trait Problem
where
    Self: Sized,
{
    /// Meta, SubtaskMeta, Task 里最好只包含基本类型数据和 Key 类型的数据
    /// 可以用 derive 做 type safe
    /// Meta 包含题目的全局配置，例如 checker，validator，时空限制等等
    type Meta: FsStore;
    /// SubtaskMeta 用于在子任务评测时覆盖全面配置（例如单独的时空限制）
    type SubtaskMeta: FsStore + crate::Override<Self::Meta>;
    /// 描述一个测试点的评测任务，可以计算 Hash 值
    type Task: FsStore;

    fn data(&self) -> Result<Data<Self::Task, Self::Meta, Self::SubtaskMeta>, Error>;
}

pub trait JudgeProblem
where
    Self: Problem,
{
    type SubmKey: Sized + ToString;
    type Subm: FsStore;

    /// 评测该任务，self 表示任务本身的信息
    fn judge_task(
        &self,
        judger: impl judger::Judger,
        meta: &Self::Meta,
        task: &Self::Task,
        subm: Self::Subm,
    ) -> Result<judger::TaskReport, Error>;
}

pub mod traditional {
    use std::io;

    use crate::data::{tempdir_unzip, StoreFile};
    use store::{FsStore, Handle};
    use tempfile::TempDir;

    use crate::{data::Data, Error};

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

    /// 传统题
    pub struct Traditional {
        dir: TempDir,
    }

    impl super::Problem for Traditional {
        type Meta = Meta;
        type SubtaskMeta = ();
        type Task = Task;

        fn data(&self) -> Result<Data<Self::Task, Self::Meta, Self::SubtaskMeta>, Error> {
            let ctx = Handle::new(self.dir.path());
            Ok(Data::open(ctx)?)
        }
    }
    impl Traditional {
        pub fn from_zip(reader: impl io::Read + io::Seek) -> Result<Self, Error> {
            Ok(Self {
                dir: tempdir_unzip(reader)?,
            })
        }
    }
}
