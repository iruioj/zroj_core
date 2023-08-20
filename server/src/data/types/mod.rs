//! 自定义的（SQL）类型，目的是利用类型的限制来防止不合法数据的出现，同时可以提供更多类型数据的存储

#[cfg(feature = "mysql")]
use diesel::{sql_types::*, *};

use serde::{Deserialize, Serialize};

mod email;
mod gender;
mod username;
mod datetime;
mod json_str;

pub use datetime::DateTime;
pub use email::EmailAddress;
pub use gender::{Gender, GenderInner};
pub use username::Username;
pub use json_str::JsonStr;
