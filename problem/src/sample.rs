use judger::{StoreFile, StoreBytes};

use crate::{data::OJData, render_data::statement::IOData, ProblemFullData, StandardProblem};

pub fn a_plus_b_statment() -> crate::render_data::Statement {
    crate::render_data::Statement {
        title: "A + B Problem".into(),
        statement: crate::render_data::statement::Inner::Standard {
            legend: r#"这是一道简单题。

你需要从标准输入中读入 $a, b$，请你输出 $a + b$。"#
                .into(),
            input_format: "输入数据包括两行，每行一个数，分别表示 $a$ 和 $b$。".into(),
            output_format: "一行一个整数表示答案。".into(),
            notes: r#"```cpp
#include<iostream>

using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
```"#
                .into(),
            samples: vec![(
                IOData {
                    fd: crate::render_data::FileDescriptor::Stdin,
                    content: "1\n2".into(),
                },
                IOData {
                    fd: crate::render_data::FileDescriptor::Stdout,
                    content: "3".into(),
                },
            )],
        },
        meta: crate::render_data::statement::StmtMeta {
            time: None,
            memory: None,
            kind: Some(crate::render_data::ProblemKind::Traditional(
                crate::render_data::IOKind::StdIO,
            )),
        },
    }
}

fn gen_a_plus_b_task(a: i32, b: i32) -> crate::prelude::TraditionalTask {
    crate::prelude::TraditionalTask {
        input: StoreFile::from_str(format!("{a} {b}"), judger::FileType::Plain),
        output: StoreFile::from_str((a + b).to_string(), judger::FileType::Plain),
    }
}

pub fn a_plus_b_data() -> StandardProblem {
    StandardProblem::Traditional(
        OJData::new(crate::prelude::TraditionalMeta {
            checker: crate::Checker::AutoCmp {
                float_relative_eps: 0.0,
                float_absoulte_eps: 0.0,
            },
            time_limit: crate::Elapse::from(1000u64),
            memory_limit: crate::Memory::from(256u64 << 20),
            output_limit: crate::Memory::from(64u64 << 20),
        })
        .set_data(crate::data::Taskset::Subtasks {
            subtasks: vec![
                crate::data::Subtask {
                    tasks: vec![
                        gen_a_plus_b_task(1, 2),
                        gen_a_plus_b_task(10, 20),
                        gen_a_plus_b_task(100, 200),
                        gen_a_plus_b_task(1000, 2000),
                        gen_a_plus_b_task(10000, 20000),
                    ],
                    meta: (),
                    score: 0.5,
                },
                crate::data::Subtask {
                    tasks: vec![
                        gen_a_plus_b_task(-100, 200),
                        gen_a_plus_b_task(-1000, 2000),
                        gen_a_plus_b_task(-10000, 20000),
                    ],
                    meta: (),
                    score: 0.3,
                },
                crate::data::Subtask {
                    tasks: vec![gen_a_plus_b_task(-10000, -20000)],
                    meta: (),
                    score: 0.2,
                },
            ],
            deps: vec![crate::data::DepRelation::new(2, 1)],
        })
        .set_pre(crate::data::Taskset::Tests {
            tasks: vec![
                gen_a_plus_b_task(1, 2),
                gen_a_plus_b_task(10, 20),
                gen_a_plus_b_task(-100, 200),
            ],
        }),
    )
}

/// A+B Problem 数据
pub fn a_plus_b_full() -> ProblemFullData {
    ProblemFullData {
        data: a_plus_b_data(),
        statement: a_plus_b_statment(),
        tutorial: crate::render_data::Tutorial {
            tutorial: crate::render_data::tutorial::Inner::Source("题解已经写在题面中".into()),
            meta: crate::render_data::tutorial::TutrMeta {
                origin: None,
                difficulty: Some("简单".into()),
                tags: vec!["IO".into()],
            },
        },
    }
}

pub fn a_plus_b_std() -> crate::prelude::TraditionalSubm {
    crate::prelude::TraditionalSubm {
        source: StoreBytes::from_str(
            r#"
#include<iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b;
    return 0;
}
"#,
            judger::FileType::GnuCpp14O2,
        ),
    }
}

#[cfg(test)]
mod tests {
    use judger::DefaultJudger;
    use store::Handle;

    use crate::{
        judger_framework,
        prelude::*,
    };

    use super::*;

    #[test]
    fn test_a_plus_b() {
        let dir = tempfile::tempdir().unwrap();
        let data = a_plus_b_data();
        let StandardProblem::Traditional(data) = data;
        let mut data = data.into_triple().1;

        // let mut data: JudgeData<_, _, _, Traditional> = JudgeData::from_data(data);

        let mut default_judger = DefaultJudger::new(Handle::new(dir.path()));

        let mut subm = TraditionalSubm {
            source: StoreBytes::from_str(
                r#"
#include<iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b;
    return 0;
}
"#,
                judger::FileType::GnuCpp14O2,
            ),
        };
        let report = judger_framework::judge::<_, _, _, Traditional>(
            &mut data,
            &mut default_judger,
            &mut subm,
        )
        .unwrap();
        dbg!(report.meta);
        drop(dir)
    }
}
