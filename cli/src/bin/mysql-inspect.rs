#![allow(non_snake_case)]
use anyhow::Context;
use diesel::{prelude::QueryableByName, Connection, RunQueryDsl};

diesel::table! {
    information_schemas_show_tables (TABLE_NAME) {
        TABLE_SCHEMA -> Text,
        TABLE_NAME -> Text,
    }
}

#[derive(QueryableByName, Debug)]
#[diesel(table_name = information_schemas_show_tables)]
#[allow(dead_code)]
struct Table {
    TABLE_SCHEMA: String,
    TABLE_NAME: String,
}

/// inspect the database of (new) ZROJ
fn main() -> anyhow::Result<()> {
    let mut conn = diesel::MysqlConnection::establish(&format!(
        "mysql://test:test@127.0.0.1:3306/information_schema"
    ))
    .context("establish connection")?;
    let r: Vec<Table> = diesel::sql_query("select TABLE_SCHEMA, TABLE_NAME from TABLES")
        .get_results(&mut conn)
        .expect("query information_schemas");
    let r: Vec<Table> = r
        .into_iter()
        .filter(|tb| tb.TABLE_SCHEMA != "information_schema")
        .collect();
    dbg!(&r);
    Ok(())
}
