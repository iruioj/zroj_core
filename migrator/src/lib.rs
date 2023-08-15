pub mod uoj;
pub mod zroj;

use diesel::{Connection, MysqlConnection};

fn establish_connection(
    user: &str,
    password: &str,
    host: &str,
    port: u32,
    dbname: &str,
) -> MysqlConnection {
    MysqlConnection::establish(&format!("mysql://{user}:{password}@{host}:{port}/{dbname}"))
        .expect("establish connection")
}
