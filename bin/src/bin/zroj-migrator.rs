#[tokio::main]
async fn main() {
    std::fs::create_dir("stmt_data").unwrap();
    let db = server::data::problem_statement::DefaultDB::new("stmt_data");
    migrator::zroj::export_problem_statements(db).await;
}
