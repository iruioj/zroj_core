use store::{FsStore, Handle};

use crate::data::{Data, OJData};

pub mod traditional;
pub type Traditional = traditional::Traditional;
pub type TraditionalOJData = OJData<traditional::Task, traditional::Meta, ()>;
pub type TraditionalData = Data<traditional::Task, traditional::Meta, ()>;

/// OJ 支持的题目类型，用于题目数据的保存和读取
pub enum StandardProblem {
    Traditional(TraditionalOJData),
}

impl StandardProblem {
    pub fn meta_description(&self) -> String {
        let info = match self {
            StandardProblem::Traditional(pr) => {
                ("traditional", pr.meta.time_limit, pr.meta.memory_limit)
            }
        };
        format!(
            "type: {}\ntl: {}\nml: {}",
            info.0,
            info.1.pretty(),
            info.2.pretty()
        )
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
