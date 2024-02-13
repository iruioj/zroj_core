mod config;
mod error;
mod prob;
mod prob_set;

pub use config::Config;
pub use error::Error;
pub use prob::Builtin;
pub use prob_set::ProblemSet;

use std::{fs, io, path::PathBuf};

/// 一个可评测的题目的数据
///
/// 其本体保存于文件系统中的某个（临时）文件
///
/// 不会涉及到对数据文件的修改。
pub trait Problem: Sized {
    /// 题目的元数据。
    ///
    /// 对于内置题目，元数据以 enum Builtin 的形式给出，这样只需要实现一个 Problem<Meta = BuiltinMeta>
    /// 即可。而如果想要定义新的题目类型，可以自己实现对应的 ProblemSet。这不无道理，
    /// 因为新形式的题目很可能与老题目的评测方式完全不同，可以作为一个新的题目集。
    /// 而如果要在一个题目集中加入新的题目类型，那就得在 Builtin 里加东西，
    /// 那么建议自己修改源代码和前后端然后编译。
    type Meta;

    /// 从一个文件读取题目
    fn open(path: &PathBuf) -> Result<Self, Error>;

    /// 该题目的文件路径
    fn path(&self) -> &PathBuf;

    /// 验证题目数据的合法性
    fn validate(&self) -> Result<(), Error>;

    fn meta(&self) -> Result<(), Self::Meta>;

    /// 不会验证 reader 里的内容是否合法
    fn replace_reader(&self, reader: &mut impl std::io::Read) -> Result<(), Error> {
        // create a file if it does not exist, and will truncate it if it does.
        let mut file = fs::File::create(self.path())?;
        io::copy(reader, &mut file)?;
        Ok(())
    }

    /// 不会验证 file 里的内容是否合法
    fn replace(&self, file: &PathBuf) -> Result<(), Error> {
        let mut this = fs::File::create(self.path())?;
        let mut file = fs::File::open(file)?;
        io::copy(&mut file, &mut this)?;
        Ok(())
    }

    /// 先验证数据的合法性再替换
    fn replace_valid(&self, file: &PathBuf) -> Result<(), Error> {
        let p = Self::open(file)?;
        p.validate()?;
        self.replace(file)
    }
}

pub mod meta {
    use serde_derive::{Deserialize, Serialize};

    /// 题目的元数据
    ///
    /// - Task: 测试数据的类型（任务类型）
    /// - SubtaskMeta: 子任务的元数据类型。
    /// 设置为多态的原因是，并非所有的题目都是以时间限制+空间限制的形式给出限定。
    /// 比如对于交互题，可以有更细致的限制；对提答题可以有文件大小限制；
    /// 对更多用户自定义的评测题目，可能限制会更多。
    /// 这些东西虽然可以在不同类型题目的 judger 中写逻辑判断来实现，但是从设计原则的角度，
    /// 写成多态有利于扩展，并且没有带来额外的开发代价。
    ///
    ///
    /// 将所有形式的题目抽象出共同特征，我们目前可以确定：
    ///
    /// - 子任务/测试点的评分模式是可以固定的（分数的计算可以自定义）
    /// - 子任务具有与整个评测任务相似的结构，除了不能有子任务
    #[derive(Serialize, Deserialize)]
    pub struct Metadata<Task, Meta, SubtaskMeta>
    where
        Task: Sized,
        Meta: Sized,
        SubtaskMeta: Sized + Override<Meta>,
    {
        /// 测试数据
        pub tasks: Taskset<Task, SubtaskMeta>,
        /// 默认的子任务元数据。
        ///
        /// 在评测时如果没有子任务就按这里的限制评测，如果有那么各自子任务的元数据会代替
        pub meta: Meta,
        /// 子任务计分规则
        pub rule: Rule,
    }

    /// 实现了 Override 的可以在默认值的基础上将一部分数据覆盖
    pub trait Override<T> {
        fn over(&self, default: &T) -> T;
    }

    impl<T: Clone> Override<T> for () {
        fn over(&self, default: &T) -> T {
            default.clone()
        }
    }

    /// 任务集合
    #[derive(Serialize, Deserialize)]
    pub enum Taskset<Task, SubtaskMeta> {
        Subtasks {
            tasks: Vec<(Task, SubtaskMeta)>,
            /// (a, b) 表示  b 依赖 a
            deps: Vec<(usize, usize)>,
        },
        Tests {
            tasks: Vec<Task>,
        },
    }

    /// 子任务记分规则
    #[derive(Serialize, Deserialize)]
    pub enum Rule {
        /// 各测试点得分和
        Sum,
        /// 取各测试点最低分
        Minimum,
    }

    pub mod builtin {
        use serde_derive::{Deserialize, Serialize};
        use std::path::PathBuf;

        /// 内置的题目数据类型
        pub enum Builtin {
            /// 传统题
            Traditional(traditional::Metadata),
            /// 交互题
            Interactive(interactive::Metadata),
            /// 提交答案
            Plain(plain::Metadata),
        }

        #[allow(dead_code)]
        #[derive(Serialize, Deserialize)]
        pub struct InOutTask {
            input: PathBuf,
            output: PathBuf,
        }
        #[derive(Serialize, Deserialize)]
        pub struct TMLimitMeta {
            time_limit: Option<u32>,
            memory_limit: Option<u32>,
        }

        mod traditional {
            use std::path::PathBuf;

            use crate::meta::Override;

            #[derive(Clone)]
            #[allow(dead_code)]
            pub struct Meta {
                checker: PathBuf,
                validator: Option<PathBuf>,
                /// 时间限制
                time_limit: u32,
                /// 空间限制
                memory_limit: u32,
            }
            impl Override<Meta> for super::TMLimitMeta {
                fn over(&self, default: &Meta) -> Meta {
                    let mut meta = default.clone();
                    if let Some(t) = self.time_limit {
                        meta.time_limit = t;
                    }
                    if let Some(t) = self.memory_limit {
                        meta.memory_limit = t;
                    }
                    meta
                }
            }
            pub type Metadata = crate::meta::Metadata<super::InOutTask, Meta, super::TMLimitMeta>;
        }
        mod interactive {
            use std::path::PathBuf;

            use crate::meta::Override;

            #[derive(Clone)]
            #[allow(dead_code)]
            pub struct Meta {
                checker: PathBuf,
                interactor: PathBuf,
                validator: Option<PathBuf>,
                /// 时间限制
                time_limit: u32,
                /// 空间限制
                memory_limit: u32,
            }
            impl Override<Meta> for super::TMLimitMeta {
                fn over(&self, default: &Meta) -> Meta {
                    let mut meta = default.clone();
                    if let Some(t) = self.time_limit {
                        meta.time_limit = t;
                    }
                    if let Some(t) = self.memory_limit {
                        meta.memory_limit = t;
                    }
                    meta
                }
            }
            pub type Metadata = crate::meta::Metadata<super::InOutTask, Meta, super::TMLimitMeta>;
        }
        mod plain {
            use std::path::PathBuf;

            #[derive(Clone)]
            #[allow(dead_code)]
            pub struct Meta {
                checker: PathBuf,
                validator: Option<PathBuf>,
            }
            pub type Metadata = crate::meta::Metadata<PathBuf, Meta, ()>;
        }
    }
}
