//! 传统题的评测

use std::path::PathBuf;

use crate::{
	lang::LangOption,
	JudgeResult,
};

pub use problem::Problem;

pub struct Submission<L: LangOption> {
	/// 提交记录对应的题目
	prob: Problem,
	/// 语言
	lang: L,
	/// 源代码相对路径
	source: PathBuf,
}

impl<L: LangOption> Submission<L> {
    pub fn new(prob: Problem, lang: L, source: PathBuf) -> Self {
        Self {
			prob,
			lang,
            source,
        }
    }
	pub fn judge(s: Submission) -> JudgeResult {
		eprintln!("{}", s.source.display());
	}
}