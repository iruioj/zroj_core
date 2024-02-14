use judger::{SourceFile, StoreFile};

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

pub fn a_plus_b_data() -> StandardProblem {
    fn gen_a_plus_b_task(a: i32, b: i32) -> crate::prelude::traditional::Task {
        crate::prelude::traditional::Task {
            input: StoreFile::from_str(format!("{a} {b}"), judger::FileType::Plain),
            output: StoreFile::from_str((a + b).to_string(), judger::FileType::Plain),
        }
    }

    StandardProblem::Traditional(
        OJData::new(crate::prelude::traditional::Meta {
            checker: crate::Checker::AutoCmp {
                float_relative_eps: 0.0,
                float_absoulte_eps: 0.0,
                to_lower_case: false,
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
                    score: 0.5,
                },
                crate::data::Subtask {
                    tasks: vec![
                        gen_a_plus_b_task(-100, 200),
                        gen_a_plus_b_task(-1000, 2000),
                        gen_a_plus_b_task(-10000, 20000),
                    ],
                    score: 0.3,
                },
                crate::data::Subtask {
                    tasks: vec![gen_a_plus_b_task(-10000, -20000)],
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

pub fn a_plus_b_std() -> crate::prelude::traditional::Subm {
    crate::prelude::traditional::Subm {
        source: SourceFile::from_str(
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

pub fn a_plus_b_wa() -> crate::prelude::traditional::Subm {
    crate::prelude::traditional::Subm {
        source: SourceFile::from_str(
            r#"
#include<iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << a - b;
    return 0;
}
"#,
            judger::FileType::GnuCpp14O2,
        ),
    }
}

pub fn quine_statment() -> crate::render_data::Statement {
    crate::render_data::Statement {
        title: "Quine".into(),
        statement: crate::render_data::statement::Inner::Standard {
            legend: r#"写一个程序，使其能输出自己的源代码。

代码中必须至少包含 $10$ 个可见字符。

"#
            .into(),
            input_format: "没有输入文件。".into(),
            output_format: "你的源代码。".into(),
            notes: r##"```cpp
#include<cstdio>
char*s="#include<cstdio>%cchar*s=%c%s%c;main(){printf(s,10,34,s,34);}";main(){printf(s,10,34,s,34);}
```"##
                .into(),
            samples: vec![],
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

pub fn quine_data() -> StandardProblem {
    StandardProblem::Traditional(
        OJData::new(crate::prelude::traditional::Meta {
            checker: crate::Checker::CABI {
                source: SourceFile::from_str(
                    include_str!("../tests/quine_checker.rs"),
                    judger::FileType::Rust,
                ),
            },
            time_limit: crate::Elapse::from(1000u64),
            memory_limit: crate::Memory::from(256u64 << 20),
            output_limit: crate::Memory::from(64u64 << 20),
        })
        .set_data(crate::data::Taskset::Tests {
            tasks: vec![crate::prelude::traditional::Task {
                input: StoreFile::from_str("", judger::FileType::Plain),
                output: StoreFile::from_str("", judger::FileType::Plain),
            }],
        }),
    )
}

pub fn quine_full() -> ProblemFullData {
    ProblemFullData {
        data: quine_data(),
        statement: quine_statment(),
        tutorial: crate::render_data::Tutorial {
            tutorial: crate::render_data::tutorial::Inner::Source(
                r##"```cpp
#include<cstdio>
char*s="#include<cstdio>%cchar*s=%c%s%c;main(){printf(s,10,34,s,34);}";main(){printf(s,10,34,s,34);}
```"##
                    .into(),
            ),
            meta: crate::render_data::tutorial::TutrMeta {
                origin: None,
                difficulty: Some("简单".into()),
                tags: vec!["SPJ".into()],
            },
        },
    }
}

pub fn quine_std() -> crate::prelude::traditional::Subm {
    crate::prelude::traditional::Subm {
        source: SourceFile::from_str(
            r##"#include<cstdio>
char*s="#include<cstdio>%cchar*s=%c%s%c;main(){printf(s,10,34,s,34);}";main(){printf(s,10,34,s,34);}"##,
            judger::FileType::GnuCpp14O2,
        ),
    }
}

#[cfg(test)]
mod tests {
    use judger::DefaultJudger;
    use store::Handle;

    use crate::{judger_framework, prelude::*};

    use super::*;

    #[test]
    fn test_sample() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let mut default_judger =
            DefaultJudger::new(Handle::new(dir.path()), Some(Handle::new(cache_dir.path())));

        // test a + b problem
        let StandardProblem::Traditional(mut data) = a_plus_b_data();
        let mut data = data.get_data_mut();

        let mut subm = a_plus_b_std();
        let report =
            judger_framework::judge::<_, _, Traditional>(&mut data, &mut default_judger, &mut subm)
                .unwrap();
        dbg!(&report);
        assert!((report.meta.score_rate - 1.).abs() < 1e-5);

        let mut subm = a_plus_b_wa();
        let report =
            judger_framework::judge::<_, _, Traditional>(&mut data, &mut default_judger, &mut subm)
                .unwrap();
        dbg!(&report);
        assert!(report.meta.score_rate.abs() < 1e-5);

        // test quine
        let StandardProblem::Traditional(mut data) = quine_data();
        let mut data = data.get_data_mut();
        let mut subm = quine_std();
        let report =
            judger_framework::judge::<_, _, Traditional>(&mut data, &mut default_judger, &mut subm)
                .unwrap();
        dbg!(&report);
        assert!((report.meta.score_rate - 1.).abs() < 1e-5);

        drop(cache_dir);
        drop(dir);
    }
}
