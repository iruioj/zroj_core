use serde::{Deserialize, Serialize};

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
    fn over(self, default: &mut T);
}

impl<T> Override<T> for () {
    fn over(self, _: &mut T) {}
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
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    /// 内置的题目数据类型
    #[derive(Serialize, Deserialize)]
    pub enum Builtin {
        /// 传统题
        Traditional(traditional::Metadata),
        // 传统题，有标准答案，用于生成输出文件，节省空间
        // TraditionalStd(traditional::Metadata),
        /// 交互题
        Interactive(interactive::Metadata),
        /// 提交答案
        Plain(plain::Metadata),
    }

    #[derive(Serialize, Deserialize)]
    pub struct InOutTask {
        pub input: PathBuf,
        pub output: PathBuf,
    }

    #[derive(Serialize, Deserialize)]
    pub struct TMLimitMeta {
        time_limit: Option<u32>,
        memory_limit: Option<u32>,
    }

    mod traditional {
        use serde::{Deserialize, Serialize};
        use std::path::PathBuf;

        use super::super::Override;

        #[derive(Serialize, Deserialize)]
        pub struct Meta {
            pub checker: PathBuf,
            pub validator: Option<PathBuf>,
            /// 时间限制
            pub time_limit: u32,
            /// 空间限制
            pub memory_limit: u32,
        }
        impl Override<Meta> for super::TMLimitMeta {
            fn over(self, meta: &mut Meta) {
                if let Some(t) = self.time_limit {
                    meta.time_limit = t;
                }
                if let Some(t) = self.memory_limit {
                    meta.memory_limit = t;
                }
            }
        }

        pub type Metadata = super::super::Metadata<super::InOutTask, Meta, super::TMLimitMeta>;
    }

    mod interactive {
        use serde::{Deserialize, Serialize};
        use std::path::PathBuf;

        use super::super::Override;

        #[derive(Serialize, Deserialize)]
        pub struct Meta {
            pub checker: PathBuf,
            pub interactor: PathBuf,
            pub validator: Option<PathBuf>,
            /// 时间限制
            pub time_limit: u32,
            /// 空间限制
            pub memory_limit: u32,
        }
        impl Override<Meta> for super::TMLimitMeta {
            fn over(self, meta: &mut Meta) {
                if let Some(t) = self.time_limit {
                    meta.time_limit = t;
                }
                if let Some(t) = self.memory_limit {
                    meta.memory_limit = t;
                }
            }
        }
        pub type Metadata = super::super::Metadata<super::InOutTask, Meta, super::TMLimitMeta>;
    }
    mod plain {
        use serde::{Deserialize, Serialize};
        use std::path::PathBuf;

        #[derive(Serialize, Deserialize)]
        pub struct Meta {
            pub checker: PathBuf,
            pub validator: Option<PathBuf>,
        }
        pub type Metadata = super::super::Metadata<PathBuf, Meta, ()>;
    }
}
