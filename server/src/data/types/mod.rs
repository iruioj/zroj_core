#[cfg(feature = "mysql")]
use diesel::{sql_types::*, *};

use serde::{Deserialize, Serialize};

mod email;
mod gender;
mod username;
mod datetime;

pub use datetime::DateTime;
pub use email::EmailAddress;
pub use gender::Gender;
pub use username::Username;

