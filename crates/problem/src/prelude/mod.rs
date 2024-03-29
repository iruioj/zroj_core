use store::{FsStore, Handle};

use crate::data::OJData;

pub mod traditional;
pub type Traditional = traditional::Traditional;
type TraditionalOJData = OJData<traditional::Task, traditional::Meta>;

/// OJ 支持的题目类型，用于题目数据的保存和读取
#[non_exhaustive]
pub enum StandardProblem {
    Traditional(TraditionalOJData),
}

impl std::fmt::Debug for StandardProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            Self::Traditional(arg0) => {
                write!(f, "[traditional] ")?;
                arg0
            }
        };
        inner.fmt(f)
    }
}

/// 手动实现 FsStore 以保证向下兼容
impl FsStore for StandardProblem {
    fn open(ctx: &Handle) -> Result<Self, store::Error> {
        if ctx.join("traditional").path().exists() {
            Ok(Self::Traditional(TraditionalOJData::open(
                &ctx.join("traditional"),
            )?))
        } else {
            Err(anyhow::anyhow!("invalid problem"))?
        }
    }

    fn save(&mut self, ctx: &Handle) -> Result<(), store::Error> {
        match self {
            StandardProblem::Traditional(t) => t.save(&ctx.join("traditional")),
        }
    }
}
