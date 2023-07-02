use std::collections::HashMap;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "uoj_config.pest"] // relative to src
struct ConfigParser;

#[derive(Debug)]
pub enum ParseError {
    SyntaxError(Box<dyn std::error::Error>),
    ParseStrError(Box<dyn std::error::Error>),
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::SyntaxError(e) => write!(f, "error parsing syntax: {e}"),
            ParseError::ParseStrError(e) => write!(f, "error parsing str: {e}"),
        }
    }
}
impl std::error::Error for ParseError {}
#[derive(Debug)]
pub struct Config {
    pub use_builtin_judger: bool,
    pub use_builtin_checker: Option<String>,
    pub n_tests: u32,
    pub n_ex_tests: u32,
    pub n_sample_tests: u32,
    pub input_pre: String,
    pub input_suf: String,
    pub output_pre: String,
    pub output_suf: String,
    pub time_limit: u32,
    pub memory_limit: u32,
    pub output_limit: u32,
}
pub fn parse_config(content: &str) -> Result<Config, ParseError> {
    let mut r = ConfigParser::parse(Rule::file, content)
        .map_err(|e| ParseError::SyntaxError(Box::new(e)))?;
    let mut map = HashMap::new();
    let file = r.find(|i| Rule::file == i.as_rule()).unwrap();
    for item in file.into_inner().filter(|i| i.as_rule() != Rule::EOI) {
        if let Rule::line = item.as_rule() {
            dbg!(&item);
            let mut it = item
                .as_span()
                .as_str()
                .split_whitespace()
                .map(|s| s.to_string());
            // not empty line
            if let Some(key) = it.next() {
                dbg!(&key);
                let value: Vec<String> = it.collect();
                map.insert(key, value);
            }
        } else {
            panic!("invalid item (rule: {:?})", item.as_rule())
        }
    }
    dbg!(&map);

    let get_str = |key: &str| map.get(key).map(|s| s[0].clone());
    let get_u32 = |key: &str, default: u32| {
        get_str(key)
            .map(|s| s.parse())
            .unwrap_or(Ok(default))
            .map_err(|e| ParseError::ParseStrError(Box::new(e)))
    };

    Ok(Config {
        use_builtin_judger: map
            .get("use_builtin_judger")
            .map(|s| s[0] == "on")
            .unwrap_or(false),
        use_builtin_checker: get_str("use_builtin_checker"),
        n_tests: get_u32("n_tests", 0)?,
        n_ex_tests: get_u32("n_ex_tests", 0)?,
        n_sample_tests: get_u32("n_sample_tests", 0)?,
        input_pre: get_str("input_pre").unwrap(),
        input_suf: get_str("input_suf").unwrap(),
        output_pre: get_str("output_pre").unwrap(),
        output_suf: get_str("output_suf").unwrap(),
        time_limit: get_u32("time_limit", 0)?,
        memory_limit: get_u32("memory_limit", 0)?,
        output_limit: get_u32("output_limit", 0)?,
    })
}
