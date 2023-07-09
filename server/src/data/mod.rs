pub mod error;
pub mod group;
pub mod types;
pub mod user;
pub mod problem_statement;

pub mod problem_config;
pub mod schema;


/// 定义一个类型为 web::Data<ty> 的值
#[macro_export]
macro_rules! mkdata {
    ($t:ty, $e:expr) => {
        actix_web::web::Data::from(std::sync::Arc::new($e) as std::sync::Arc<$t>)
    };
}
pub use mkdata;