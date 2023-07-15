use judger::{
    sandbox::{Elapse, Memory},
    Status, TaskMeta, SCOER_EPS,
};

use crate::data::Rule;

pub struct JudgeMonitor {
    status: Status,
    time: Elapse,
    memory: Memory,
    score: f64,
    rule: Rule,
    default_score: f64,
}

impl JudgeMonitor {
    pub fn new(default_score: f64, rule: &Rule) -> Self {
        Self {
            status: Status::Accepted,
            time: 0.into(),
            memory: 0.into(),
            score: match rule {
                Rule::Sum => 0.0,
                Rule::Minimum => 1.0,
            },
            rule: rule.clone(),
            default_score,
        }
    }
    pub fn update(&mut self, r: &TaskMeta) {
        self.status.update(r.status.clone());
        self.time = self.time.max(r.time);
        self.memory = self.memory.max(r.memory);
        let score = r.status.score_rate() * r.status.total_score().unwrap_or(self.default_score);
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
            score: self.score,
            status: self.status.clone(),
            time: self.time,
            memory: self.memory,
        }
    }
}
