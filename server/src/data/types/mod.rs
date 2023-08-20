//! 自定义的（SQL）类型，目的是利用类型的限制来防止不合法数据的出现，同时可以提供更多类型数据的存储

#[cfg(feature = "mysql")]
use diesel::{sql_types::*, *};

use serde::{Deserialize, Serialize};

mod datetime;
mod email;
mod gender;
mod json_str;
mod username;

pub use datetime::DateTime;
pub use email::EmailAddress;
pub use gender::{Gender, GenderInner};
pub use json_str::JsonStr;
pub use username::Username;
