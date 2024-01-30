#![allow(non_snake_case)]
use diesel::{prelude::QueryableByName, RunQueryDsl};

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
fn main() {
    let mut conn =
        migrator::establish_connection("test", "test", "127.0.0.1", 3306, "information_schema");
    let r: Vec<Table> = diesel::sql_query("select TABLE_SCHEMA, TABLE_NAME from TABLES")
        .get_results(&mut conn)
        .expect("query information_schemas");
    let r: Vec<Table> = r
        .into_iter()
        .filter(|tb| tb.TABLE_SCHEMA != "information_schema")
        .collect();
    dbg!(&r);
}
