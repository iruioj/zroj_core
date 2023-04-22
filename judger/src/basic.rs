//! 传统题的评测

use std::path::PathBuf;

use crate::{
	lang::LangOption,
	JudgeResult,
};

pub use problem::Builtin;

pub struct Submission<L: LangOption> {
	/// 提交记录对应的题目
	prob: Builtin,
	/// 语言
	lang: L,
	/// 源代码相对路径
	source: PathBuf,
}

impl<L: LangOption> Submission<L> {
    pub fn new(prob: Builtin, lang: L, source: PathBuf) -> Self {
        Self {
			prob,
			lang,
            source,
        }
    }
	pub fn judge(&self) -> JudgeResult {
		eprintln!("{}", self.source.display());
		todo!()
	}
}