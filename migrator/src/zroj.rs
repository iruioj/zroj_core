//! 这部分可能是 zroj 自己魔改的东西，所以不归 uoj

use crate::establish_connection;
use diesel::prelude::*;
use problem::render_data::{
    statement::{Inner, StmtMeta},
    Statement,
};

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

pub async fn export_problem_statements(db: impl server::data::problem_statement::Manager) {
    eprintln!("try connecting to db");
    let mut conn = establish_connection("root", "Zhengruioi2333", "127.0.0.1", 33060, "zroi");
    eprintln!("connection established");

    let r: Vec<(i32, String, Option<String>)> = problems::table
        .left_join(problems_contents::table)
        .select((
            problems::id,
            problems::title,
            problems_contents::statement_md.nullable(),
        ))
        .limit(200)
        .load::<(i32, String, Option<String>)>(&mut conn)
        .expect("query problems_contents");

    for (id, title, stmt) in r {
        db.insert(
            id as u32,
            Statement {
                statement: Inner::Legacy(stmt.unwrap_or_default()),
                meta: StmtMeta {
                    title,
                    time: None,
                    memory: None,
                    kind: None,
                },
            },
        )
        .await
        .unwrap();
    }
}
