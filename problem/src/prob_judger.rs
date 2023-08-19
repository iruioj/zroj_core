use judger::{
    sandbox::{Elapse, Memory},
    Status, TaskMeta, SCOER_EPS,
};

use crate::data::Rule;

pub struct Summarizer {
    status: Status,
    time: Elapse,
    memory: Memory,
    score: f64,
    rule: Rule,
}

impl Summarizer {
    pub fn new(rule: Rule) -> Self {
        Self {
            status: Status::Good,
            time: 0.into(),
            memory: 0.into(),
            score: match rule {
                Rule::Sum => 0.0,
                Rule::Minimum => 1.0,
            },
            rule: rule.clone(),
        }
    }
    pub fn update(&mut self, r: &TaskMeta, task_score: f64) {
        self.status.update(r.status.clone());
        self.time = self.time.max(r.time);
        self.memory = self.memory.max(r.memory);
        let score = r.score_rate * task_score;
        self.score = match self.rule {
            Rule::Sum => self.score + score,
            Rule::Minimum => self.score.min(score),
        }
    }
    pub fn skippable(&self) -> bool {
        if matches!(self.rule, Rule::Minimum) && self.score < SCOER_EPS {
            return true;
        }
        false
    }
    pub fn report(&self) -> TaskMeta {
        TaskMeta {
            score_rate: self.score,
            status: self.status.clone(),
            time: self.time,
            memory: self.memory,
        }
    }
}
