use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Context};
use pest::Parser;
use pest_derive::Parser;
use problem::{
    data::{DepRelation, FileType, StoreFile, Subtask, Taskset},
    prelude::*,
    Checker,
};

#[derive(Parser)]
#[grammar = "uoj_config.pest"] // relative to src
struct ConfigParser;

#[derive(Debug)]
pub struct SubtaskConfig {
    start: u64,
    end: u64,
    score: u64,
}

#[derive(Debug)]
pub struct Config {
    use_builtin_judger: bool,
    with_implementer: bool,
    use_builtin_checker: Option<String>,
    n_tests: Option<u64>,
    n_ex_tests: Option<u64>,
    n_sample_tests: Option<u64>,
    input_pre: String,
    input_suf: String,
    output_pre: String,
    output_suf: String,
    time_limit: Option<u64>,
    memory_limit: Option<u64>,
    output_limit: Option<u64>,
    subtasks: Option<(BTreeMap<u64, SubtaskConfig>, Vec<(u64, u64)>)>,
}
pub fn parse_config(content: &str) -> anyhow::Result<Config> {
    let mut r =
        ConfigParser::parse(Rule::file, content).map_err(|e| anyhow!("syntax error: {e}"))?;
    let mut map = HashMap::new();
    let file = r.find(|i| Rule::file == i.as_rule()).unwrap();
    for item in file.into_inner().filter(|i| i.as_rule() != Rule::EOI) {
        if Rule::line != item.as_rule() {
            return Err(anyhow!("invalid item (rule: {:?})", item.as_rule()));
        }
        let mut it = item
            .as_span()
            .as_str()
            .split_whitespace()
            .map(|s| s.to_string());
        // not empty line
        if let Some(key) = it.next() {
            let value: Vec<String> = it.collect();
            map.insert(key, value);
        }
    }

    let get_str = |key: &str| map.get(key).map(|s| s[0].clone());
    let get_u64 = |key: &str| {
        get_str(key)
            .map(|s| s.parse())
            .map_or(Ok(None), |v| v.map(Some))
            // .unwrap_or(Ok(default))
            .map_err(|e| anyhow!("parse string error: {e}"))
    };
    let n_subtasks = get_u64("n_subtasks")?;

    Ok(Config {
        use_builtin_judger: map
            .get("use_builtin_judger")
            .map(|s| s[0] == "on")
            .unwrap_or(false),
        with_implementer: map
            .get("with_implementer")
            .map(|s| s[0] == "on")
            .unwrap_or(false),
        use_builtin_checker: get_str("use_builtin_checker"),
        n_tests: get_u64("n_tests")?,
        n_ex_tests: get_u64("n_ex_tests")?,
        n_sample_tests: get_u64("n_sample_tests")?,
        input_pre: get_str("input_pre").unwrap(),
        input_suf: get_str("input_suf").unwrap(),
        output_pre: get_str("output_pre").unwrap(),
        output_suf: get_str("output_suf").unwrap(),
        time_limit: get_u64("time_limit")?,
        memory_limit: get_u64("memory_limit")?,
        output_limit: get_u64("output_limit")?,
        subtasks: n_subtasks
            .map(|n_subtasks| {
                Ok((
                    (1..=n_subtasks).try_fold::<_, _, anyhow::Result<_>>(
                        BTreeMap::<u64, SubtaskConfig>::new(),
                        |mut v, i| {
                            v.insert(
                                i,
                                SubtaskConfig {
                                    start: v.get(&(i - 1)).map(|o| o.end).unwrap_or(0) + 1,
                                    end: get_u64(&format!("subtask_end_{i}"))?.unwrap(),
                                    score: get_u64(&format!("subtask_score_{i}"))?.unwrap(),
                                },
                            );
                            Ok(v)
                        },
                    )?,
                    map.iter()
                        .filter_map(|(k, v)| {
                            if k.contains("subtask_dependence_") {
                                Some((
                                    k["subtask_dependence_".len()..].parse::<u64>().unwrap(),
                                    v[0].parse::<u64>().unwrap(),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect(),
                ))
            })
            .map_or(Ok(None), |v: anyhow::Result<_>| v.map(Some))?,
    })
}

impl Config {
    fn _get_task(
        &self,
        dir: &store::Handle,
        prefix: &str,
        cur: u64,
    ) -> anyhow::Result<traditional::Task> {
        Ok(traditional::Task {
            input: StoreFile {
                file: dir
                    .join(format!(
                        "{prefix}{}{cur}.{}",
                        self.input_pre, self.input_suf
                    ))
                    .open_file()
                    .context("open input file")?,
                file_type: FileType::Plain,
            },
            output: StoreFile {
                file: dir
                    .join(format!(
                        "{prefix}{}{cur}.{}",
                        self.output_pre, self.output_suf
                    ))
                    .open_file()
                    .context("open output file")?,
                file_type: FileType::Plain,
            },
        })
    }
    fn get_task(&self, dir: &store::Handle, cur: u64) -> anyhow::Result<traditional::Task> {
        self._get_task(dir, "", cur)
    }
    fn get_ex_task(&self, dir: &store::Handle, cur: u64) -> anyhow::Result<traditional::Task> {
        self._get_task(dir, "ex_", cur)
    }
}

pub fn load_data(dir: store::Handle) -> anyhow::Result<TraditionalOJData> {
    let conf = parse_config(std::fs::read_to_string(dir.join("problem.conf").path())?.as_str())?;
    if !conf.use_builtin_judger {
        return Err(anyhow!("not builtin judger"));
    }
    if conf.with_implementer {
        return Err(anyhow!("use custom implementer"));
    }
    let mut ojdata = TraditionalOJData::new(problem::prelude::traditional::Meta {
        checker: if let Some(checker) = &conf.use_builtin_checker {
            if checker == "ncmp" {
                Checker::AutoCmp {
                    float_relative_eps: 0.0,
                    float_absoulte_eps: 0.0,
                }
            }
            // default checker
            else {
                Checker::FileCmp
            }
        } else {
            // TODO: spj
            unimplemented!()
        },
        time_limit: conf.time_limit.unwrap_or(5000).into(),
        memory_limit: conf.memory_limit.unwrap_or(256 << 20).into(),
        output_limit: conf.output_limit.unwrap_or(64 << 20).into(),
    })
    .set_data(if let Some((subtasks, deps)) = &conf.subtasks {
        Taskset::Subtasks {
            subtasks: subtasks
                .iter()
                .map(|(_k, v)| {
                    Ok(Subtask {
                        tasks: (v.start..=v.end)
                            .map(|cur| conf.get_task(&dir, cur))
                            .collect::<anyhow::Result<_>>()?,
                        meta: (),
                        score: v.score as f64 / 100.0,
                    })
                })
                .collect::<anyhow::Result<_>>()?,
            deps: deps
                .iter()
                .map(|(a, b)| DepRelation::new(*a as usize - 1, *b as usize - 1))
                .collect(),
        }
    } else {
        Taskset::Tests {
            tasks: (1..=conf.n_tests.unwrap())
                .map(|cur| conf.get_task(&dir, cur))
                .collect::<Result<Vec<_>, _>>()?,
        }
    });
    if let Some(n_sample_tests) = conf.n_sample_tests {
        ojdata = ojdata.set_pre(Taskset::Tests {
            tasks: (1..=n_sample_tests)
                .map(|cur| conf.get_ex_task(&dir, cur))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
    if let Some(n_ex_tests) = conf.n_ex_tests {
        ojdata = ojdata.set_pre(Taskset::Tests {
            tasks: (conf.n_sample_tests.unwrap_or(0) + 1..=n_ex_tests)
                .map(|cur| conf.get_ex_task(&dir, cur))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    Ok(ojdata)
}
