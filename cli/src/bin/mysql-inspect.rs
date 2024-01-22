#![allow(non_snake_case)]
use diesel::{prelude::QueryableByName, RunQueryDsl};

diesel::table! {
    information_schemas_show_tables (Tables_in_information_schema) {
        Tables_in_information_schema -> Text
    }
}

#[derive(QueryableByName, Debug)]
#[diesel(table_name = information_schemas_show_tables)]
#[allow(dead_code)]
struct Table {
    Tables_in_information_schema: String,
}

fn main() {
    let mut conn =
        migrator::establish_connection("test", "test", "127.0.0.1", 3306, "information_schema");
    let r: Vec<Table> = diesel::sql_query("SHOW tables")
        .get_results(&mut conn)
        .expect("query information_schemas");
    dbg!(&r);
}
