use store::{FsStore, Handle};

use crate::data::{Data, OJData};

pub mod traditional;
pub type TraditionalOJData = OJData<traditional::Task, traditional::Meta, ()>;
pub type TraditionalData = Data<traditional::Task, traditional::Meta, ()>;

/// OJ 支持的题目类型，用于题目数据的保存和读取
pub enum StandardProblem {
    Traditional(TraditionalOJData),
}

/// 手动实现 FsStore 以保证向下兼容
impl FsStore for StandardProblem {
    fn open(ctx: &Handle) -> Result<Self, store::Error> {
        if ctx.join("traditional").as_ref().exists() {
            Ok(Self::Traditional(TraditionalOJData::open(
                &ctx.join("traditional"),
            )?))
        } else {
            Err(store::Error::NotFile)
        }
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), store::Error> {
        match self {
            StandardProblem::Traditional(t) => t.save(&ctx.join("traditional")),
        }
    }
}
