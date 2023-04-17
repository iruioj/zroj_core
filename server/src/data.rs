pub mod user;
pub mod schema;
#[cfg(not(feature="mysql"))]
pub type UserDataManagerType = user::hashmap::HashMap;
#[cfg(feature="mysql")]
pub type UserDataManagerType = user::database::Database;