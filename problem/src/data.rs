//! 题目数据存储的抽象

use std::io::{self, Seek, Write};

use serde::{Deserialize, Serialize};
use store::{FsStore, Handle};
use tempfile::TempDir;

use crate::{Error, Override};

// 题目的评测数据
// pub trait Data {
//     type Meta: Send;
//     type Task: Send;
//     type SubtaskMeta: Override<Self::Meta>;

//     /// 测试数据
//     fn tasks(&self)-> Taskset<Self::Task, Self::SubtaskMeta>;

// }

/// 题目的数据
///
/// - T: Task, 测试数据的类型（任务类型）
/// - M: Meta, 元数据类型，例如时空限制，checker 等等
/// - S: SubtaskMeta, 子任务的元数据类型，用于覆盖默认限制。
///
/// 设置为多态的原因是，并非所有的题目都是以时间限制+空间限制的形式给出限定。
/// 比如对于交互题，可以有更细致的限制；对提答题可以有文件大小限制；
/// 对更多用户自定义的评测题目，可能限制会更多。
/// 这些东西虽然可以在不同类型题目的 judger 中写逻辑判断来实现，但是从设计原则的角度，
/// 写成多态有利于扩展，并且没有带来额外的开发代价。
///
/// 将所有形式的题目抽象出共同特征，我们目前可以确定：
///
/// - 子任务/测试点的评分模式是可以固定的（分数的计算可以自定义）
/// - 子任务具有与整个评测任务相似的结构，除了不能有子任务
#[derive(FsStore)]
pub struct Data<T, M, S>
where
    T: FsStore,
    M: FsStore,
    S: Override<M> + FsStore,
{
    /// 测试数据
    pub tasks: Taskset<T, S>,
    /// 默认的子任务元数据。
    ///
    /// 在评测时如果没有子任务就按这里的限制评测，如果有那么各自子任务的元数据会代替
    pub meta: M,
    /// 子任务计分规则
    pub rule: Rule,
}

/// 任务集合
pub enum Taskset<Task, SubtaskMeta> {
    Subtasks {
        tasks: Vec<(Task, SubtaskMeta)>,
        /// (a, b) 表示  b 依赖 a
        deps: DepOption,
    },
    Tests {
        tasks: Vec<Task>,
    },
}

type DepOption = Vec<(usize, usize)>;

#[derive(FsStore, Serialize, Deserialize)]
struct TasksetMeta {
    /// if not none, it is subtask
    #[meta]
    subtask: Option<DepOption>,
    #[meta]
    n_tests: usize,
}

impl<Task: FsStore, SubtaskMeta: FsStore> FsStore for Taskset<Task, SubtaskMeta> {
    fn open(ctx: Handle) -> Result<Self, store::Error> {
        let meta: TasksetMeta = ctx.join("_taskset_meta").deserialize()?;
        if let Some(deps) = meta.subtask {
            Ok(Taskset::Subtasks {
                tasks: FsStore::open(ctx.join("tasks"))?,
                deps,
            })
        } else {
            Ok(Taskset::Tests {
                tasks: FsStore::open(ctx.join("tasks"))?,
            })
        }
    }

    fn save(&mut self, ctx: Handle) -> Result<(), store::Error> {
        match self {
            Taskset::Subtasks { tasks, deps } => {
                ctx.join("_taskset_meta").serialize_new_file(&TasksetMeta {
                    subtask: Some(deps.clone()),
                    n_tests: tasks.len(),
                })?;
                tasks.save(ctx.join("tasks"))?;
            }
            Taskset::Tests { tasks } => {
                ctx.join("_taskset_meta").serialize_new_file(&TasksetMeta {
                    subtask: None,
                    n_tests: tasks.len(),
                })?;
                tasks.save(ctx.join("tasks"))?;
            }
        }
        Ok(())
    }
}

/// 子任务记分规则
#[derive(Serialize, Deserialize)]
pub enum Rule {
    /// 各测试点得分和
    Sum,
    /// 取各测试点最低分
    Minimum,
}

impl FsStore for Rule {
    fn open(path: Handle) -> Result<Self, store::Error> {
        path.join("_rule").deserialize()
    }

    fn save(&mut self, path: Handle) -> Result<(), store::Error> {
        path.join("_rule").serialize_new_file(&self)
    }
}

/// 将文件解压到临时文件夹中
pub fn tempdir_unzip(reader: impl io::Read + io::Seek) -> Result<TempDir, Error> {
    let dir = TempDir::new()?;
    let mut zip = zip::ZipArchive::new(reader)?;
    zip.extract(dir.path())?;
    Ok(dir)
}

/// 一个带类型的文件
#[derive(FsStore)]
pub struct StoreFile {
    pub file: std::fs::File,
    #[meta]
    pub file_type: judger::FileType,
}

impl StoreFile {
    pub fn reset_cursor(&mut self) -> Result<(), std::io::Error> {
        self.file.seek(io::SeekFrom::Start(0))?;
        Ok(())
    }
    pub fn copy_all(&mut self, dest: &mut impl Write) -> Result<(), std::io::Error> {
        self.reset_cursor()?;
        std::io::copy(&mut self.file, dest)?;
        Ok(())
    }
}

// impl io::Read for StoreFile {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.file.read(buf)
//     }
// }
// impl io::Seek for StoreFile {
//     fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
//         self.file.seek(pos)
//     }
// }
// impl io::Write for StoreFile {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         self.file.write(buf)
//     }

//     fn flush(&mut self) -> io::Result<()> {
//         self.file.flush()
//     }
// }
