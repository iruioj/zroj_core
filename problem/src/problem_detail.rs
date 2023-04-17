use std::{path::PathBuf, fs::File};

use serde_derive::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::{config::{ProblemConfig, Checkable}, problem_set::ProblemSet};


#[derive(Serialize, Deserialize)]
pub struct Pair (PathBuf, PathBuf);

#[derive(Serialize, Deserialize)]
pub struct Single (PathBuf);

impl Checkable for Pair {
    fn check(&self, zip: &mut ZipArchive<&File>) -> bool {
        ProblemSet::validate_path(zip, &self.0) && ProblemSet::validate_path(zip, &self.1)
    }
}

impl Checkable for Single {
    fn check(&self, zip: &mut ZipArchive<&File>) -> bool {
        ProblemSet::validate_path(zip, &self.0)
    }
}

/// 题目类型，分别对应传统题、交互题、提交答案题
#[derive(Serialize, Deserialize)]
pub enum ProblemDetail {
    Traditional(ProblemConfig<Pair>), 
    Interactive(ProblemConfig<Pair>), 
    AnswerOnly(ProblemConfig<Single>)
}
