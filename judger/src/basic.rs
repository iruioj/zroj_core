//! 传统题的评测

use std::{path::PathBuf};

use problem::Builtin as Problem;

use crate::{lang::LangOption, JudgeReport, Cache, error::Error};

pub struct Submission<L: LangOption> {
	prob: Problem,
	lang: L,
	source: PathBuf,
}

impl<L: LangOption> Submission<L> {
    pub fn new(prob: Problem, lang: L, source: PathBuf) -> Self {
        return Self { prob, lang, source };
    }
	/// 评测需传入缓存系统的可变引用
	pub fn judge_traditional(&self, cache: &mut Cache) -> Result<JudgeReport, Error> {
		let _exec = cache.get_exec(&self.lang, &self.source);
		todo!();
	}
}