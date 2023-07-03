use migrator::uoj::load_data;
use store::Handle;

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

const CONFIG_CONTENT_SUBTASK: &str = r#"n_tests 36
n_ex_tests 3
n_sample_tests 3
input_pre rag
input_suf in
output_pre rag
output_suf out
time_limit 1
memory_limit 512
output_limit 64
use_builtin_judger on
use_builtin_checker ncmp
n_subtasks 4
subtask_end_1 10
subtask_score_1 10
subtask_end_2 16
subtask_score_2 10
subtask_end_3 22
subtask_score_3 30
subtask_end_4 36
subtask_score_4 50
subtask_dependence_4 3

"#;

#[test]
fn test_uoj() {
    let config = migrator::uoj::parse_config(CONFIG_CONTENT_SUBTASK).unwrap();

    dbg!(&config);
    // let data = load_data(&config, Handle::new("tests/testdata/1676")).unwrap();

    // dbg!(data);
    // data.save(Handle::new("target/save_data")).unwrap();
}
