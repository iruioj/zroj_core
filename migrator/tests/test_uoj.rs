const CONFIG_CONTENT: &str = r#"use_builtin_judger on
use_builtin_checker ncmp
n_tests 10
n_ex_tests 18
n_sample_tests 1
input_pre ab
input_suf in

output_pre ab
output_suf out
time_limit 1


memory_limit 512
output_limit 64



"#;
#[test]
fn test_uoj() {
    let config = migrator::uoj::parse_config(CONFIG_CONTENT).unwrap();
    dbg!(config);
}
