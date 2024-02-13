//! 题目数据存储

pub use judger::FileType;
use serde::{Deserialize, Serialize};
use store::FsStore;

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
#[derive(Debug)]
pub struct Data<'d, T, M>
where
    T: FsStore,
{
    /// 测试数据
    pub tasks: &'d mut Taskset<T>,
    /// 默认的子任务元数据。
    ///
    /// 在评测时如果没有子任务就按这里的限制评测，如果有那么各自子任务的元数据会代替
    pub meta: &'d mut M,
    // 子任务计分规则
    // pub rule: Rule,
}

/// 在线评测系统的题目数据
///
/// 比题目评测数据多了预测 (pre) 和额外测试 (extra)
#[derive(FsStore, Debug)]
pub struct OJData<T, M>
where
    T: FsStore,
    M: FsStore,
{
    /// 测试数据
    pub data: Taskset<T>,
    /// 样例评测的数据
    ///
    /// 初始化时与 data 的元信息一致，数据集为空
    pub pre: Taskset<T>,
    /// 额外的评测数据
    ///
    /// 初始化时与 data 的元信息一致，数据集为空
    pub extra: Taskset<T>,
    /// see [`Data`]
    pub meta: M,
    // see [`Data`]
    // pub rule: Rule,
}

impl<T, M> OJData<T, M>
where
    T: FsStore,
    M: FsStore,
{
    pub fn new(meta: M) -> Self {
        Self {
            data: Default::default(),
            pre: Default::default(),
            extra: Default::default(),
            meta,
            // rule,
        }
    }
    pub fn set_data(mut self, data: Taskset<T>) -> Self {
        self.data = data;
        self
    }
    pub fn set_pre(mut self, data: Taskset<T>) -> Self {
        self.pre = data;
        self
    }
    pub fn set_extra(mut self, data: Taskset<T>) -> Self {
        self.extra = data;
        self
    }
    pub fn get_data_mut(&mut self) -> Data<'_, T, M> {
        Data {
            tasks: &mut self.data,
            meta: &mut self.meta,
        }
    }
}

#[derive(FsStore, Debug)]
pub struct Subtask<Task>
where
    Task: FsStore,
{
    pub tasks: Vec<Task>,
    #[meta]
    pub score: f64,
}

/// 任务集合，分成测试点模式和子任务模式
#[derive(FsStore, Debug)]
pub enum Taskset<Task>
where
    Task: FsStore,
{
    Subtasks {
        subtasks: Vec<Subtask<Task>>,
        /// (a, b) 表示  b 依赖 a
        #[meta]
        deps: DepOption,
    },
    Tests {
        tasks: Vec<Task>,
    },
}

impl<T> Default for Taskset<T>
where
    T: FsStore,
{
    fn default() -> Self {
        Self::Tests {
            tasks: Vec::default(),
        }
    }
}

/// (a, b) a > b, a depends on b
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepRelation(usize, usize);

impl DepRelation {
    /// depender depends on dependee
    pub fn new(depender: usize, dependee: usize) -> Self {
        assert!(depender > dependee);
        Self(depender, dependee)
    }
    pub fn depender(&self) -> usize {
        self.0
    }
    pub fn dependee(&self) -> usize {
        self.1
    }
}

type DepOption = Vec<DepRelation>;

/// 子任务记分规则
#[derive(Serialize, Deserialize, FsStore, Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// 各测试点得分和
    Sum,
    /// 取各测试点最低分
    Minimum,
}

pub use judger::StoreFile;
