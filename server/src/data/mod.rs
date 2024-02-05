pub mod error;
pub mod file_system;
pub mod mysql;
pub mod types;

// database
pub mod gravatar;
pub mod problem_ojdata;
pub mod problem_statement;
pub mod submission;
pub mod user;

/// 定义一个类型为 [`actix_web::web::Data`] 的值
#[macro_export]
macro_rules! mkdata {
    ($t:ty, $e:expr) => {
        actix_web::web::Data::from(std::sync::Arc::new($e) as std::sync::Arc<$t>)
    };
}
pub use mkdata;
