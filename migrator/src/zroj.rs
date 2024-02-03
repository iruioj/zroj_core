//! 这部分可能是 zroj 自己魔改的东西，所以不归 uoj

use anyhow::Context;
use std::{collections::BTreeMap, process::Command};

use diesel::prelude::*;
use server::data::{
    mysql::{schema::*, schema_model::*},
    problem_ojdata::Manager,
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
pub fn export_problem_db(
    db: &mut server::data::mysql::MysqlDb,
    conn: &mut MysqlConnection,
    dry_run: bool,
) -> anyhow::Result<Vec<i32>> {
    println!("export problem database");
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
        .context("query problems_contents")?;

    let mut ret = Vec::default();

    // migrate problem statements
    for (id, title, req, extra_config, stmt) in &r {
        let req: Option<Vec<BTreeMap<String, String>>> = serde_json::from_str(&req).ok();
        let extra_config: Option<BTreeMap<String, String>> =
            serde_json::from_str(&extra_config).ok();
        // submitting config of traditional problem
        let can_update = if req
            == Some(vec![[
                ("name".to_string(), "answer".to_string()),
                ("type".to_string(), "source code".to_string()),
                ("file_name".to_string(), "answer.code".to_string()),
            ]
            .into()])
        {
            if extra_config.is_some_and(|c| c.contains_key("pdf_statement")) {
                false // FIXME: to be implemented
            } else {
                true
            }
        } else {
            false
        };
        if can_update {
            ret.push(*id);

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
                .context("save data to problems")?;

                db.migrate_replace(
                    problem_statements::table,
                    ProblemStatement {
                        id: *id as u32,
                        pid: *id as u32,
                        content: JsonStr(md::parse_ast(&stmt.clone().unwrap_or_default()).unwrap()),
                    },
                )
                .context("save data to problem_statements")?;
            }
        } else {
            println!("problem #{id} ({title}) is not migrated");
        }
    }
    Ok(ret)
}

pub fn export_problem_ojdata(
    db: &mut server::data::problem_ojdata::DefaultDB,
    ids: &[i32],
) -> anyhow::Result<()> {
    std::fs::create_dir_all("target/zroi_remote_root")?;

    Command::new("sshfs")
        .args(["root@zhengruioi.com:/root", "target/zroi_remote_root"])
        .output()
        .context("mount remote root")?;

    for id in ids.to_owned() {
        println!("migrate problem ojdata {id}");

        let ctx = store::Handle::new(format!("target/zroi_remote_root/prob-{id}"));
        if ctx.path().exists() {
            std::fs::remove_dir_all(ctx.path()).context("remove previous data")?;
        }

        Command::new("ssh")
            .arg("root@zhengruioi.com")
            .arg(format!("docker cp zroi:/var/uoj_data/{id} /root/prob-{id}"))
            .output()
            .context("copy data from docker to remote root")?;

        let data = crate::uoj::load_data(ctx).context("load uoj data")?;

        db.insert(id as u32, problem::StandardProblem::Traditional(data))
            .context("insert ojdata to db")?;
    }

    Command::new("fusermount")
        .args(["-u", "target/zroi_remote_root"])
        .output()
        .context("unmount remote root")?;
    Ok(())
}
