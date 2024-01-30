//! 这部分可能是 zroj 自己魔改的东西，所以不归 uoj

use std::collections::BTreeMap;

use diesel::prelude::*;
use server::data::{
    mysql::{schema::*, schema_model::*},
    types::JsonStr,
};

mod old {
    diesel::table! {
        problems_contents (id) {
            id -> Integer,
            statement -> Text,
            statement_md -> Text,
        }
    }

    // select column_name, data_type from information_schema.columns
    // where table_name = 'problems';
    diesel::table! {
        problems (id) {
            id -> Integer,
            title -> Text,
            is_hidden -> TinyInt,
            submission_requirement -> Text,
            hackable -> TinyInt,
            extra_config -> Text,
            zan -> Integer,
            ac_num -> Integer,
            submit_num -> Integer,
        }
    }

    diesel::joinable!(problems_contents -> problems (id));
    diesel::allow_tables_to_appear_in_same_query!(problems_contents, problems,);
}
pub fn export_problem_data(
    db: &mut server::data::mysql::MysqlDb,
    conn: &mut MysqlConnection,
    dry_run: bool,
) {
    let r: Vec<(i32, String, String, String, Option<String>)> = old::problems::table
        .left_join(old::problems_contents::table)
        .select((
            old::problems::id,
            old::problems::title,
            old::problems::submission_requirement,
            old::problems::extra_config,
            old::problems_contents::statement_md.nullable(),
        ))
        // .limit(200)
        .load::<(i32, String, String, String, Option<String>)>(conn)
        .expect("query problems_contents");

    // migrate problem statements
    let mut total_updated = 0;
    for (id, title, req, extra_config, stmt) in &r {
        let req: Option<Vec<BTreeMap<String, String>>> = serde_json::from_str(&req).ok();
        let extra_config: Option<BTreeMap<String, String>> =
            serde_json::from_str(&extra_config).ok();
        let can_update = if let Some(req) = req {
            if req.len() == 1 {
                // submitting config of traditional problem
                if req[0]["name"] == "answer"
                    && req[0]["type"] == "source code"
                    && req[0]["file_name"] == "answer.code"
                {
                    if let Some(extra_config) = extra_config {
                        if extra_config.contains_key("pdf_statement") {
                            false // FIXME
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        if can_update {
            total_updated += 1;

            if !dry_run {
                // 在原来的 zroj 当中每道题的题面只有一个，因此可以认为 id = pid
                db.migrate_replace(
                    problems::table,
                    Problem {
                        id: *id as u32,
                        title: title.clone(),
                        meta: JsonStr(problem::render_data::statement::StmtMeta {
                            time: None,
                            memory: None,
                            kind: Some(problem::render_data::ProblemKind::Traditional(
                                problem::render_data::IOKind::StdIO,
                            )),
                        }),
                    },
                )
                .unwrap();

                db.migrate_replace(
                    problem_statements::table,
                    ProblemStatement {
                        id: *id as u32,
                        pid: *id as u32,
                        content: JsonStr(md::parse_ast(&stmt.clone().unwrap_or_default()).unwrap()),
                    },
                )
                .unwrap();
            }
        } else {
            println!("problem #{id} ({title}) is not migrated");
        }
    }
    println!("total migrated: {total_updated}")
}
