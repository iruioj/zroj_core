use store::{FsStore, Handle};

use crate::data::OJData;

pub mod traditional;
pub type TraditionalData = OJData<traditional::Task, traditional::Meta, ()>;

/// OJ 支持的题目类型，用于题目数据的保存和读取
pub enum StandardProblem {
    Traditional(TraditionalData),
}

/// 手动实现 FsStore 以保证向下兼容
impl FsStore for StandardProblem {
    fn open(ctx: &Handle) -> Result<Self, store::Error> {
        if ctx.join("traditional").as_ref().exists() {
            Ok(Self::Traditional(TraditionalData::open(
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
