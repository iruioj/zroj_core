pub mod error;
pub mod mysql;
mod fs_store;
pub mod types;

// database
pub mod gravatar;
pub mod problem_ojdata;
pub mod problem_statement;
pub mod submission;
pub mod user;

/// 定义一个类型为 web::Data<ty> 的值
#[macro_export]
macro_rules! mkdata {
    ($t:ty, $e:expr) => {
        actix_web::web::Data::from(std::sync::Arc::new($e) as std::sync::Arc<$t>)
    };
}
pub use mkdata;

// fn notfound_as_none<T>(r: Result<T, error::DataError>) -> Result<Option<T>, error::DataError> {
//     match r {
//         Ok(t) => Ok(Some(t)),
//         Err(e) => match e {
//             error::DataError::NotFound => Ok(None),
//             #[cfg(feature = "mysql")]
//             error::DataError::Diesel(diesel::result::Error::NotFound) => Ok(None),
//             e => Err(e),
//         },
//     }
// }