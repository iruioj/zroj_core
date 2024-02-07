//! 自定义的（SQL）基础类型，目的是利用类型的限制来防止不合法数据的出现，同时可以提供更多类型数据的存储

use diesel::{sql_types::*, *};

use serde::{Deserialize, Serialize};

mod datetime;
mod email;
mod gender;
#[macro_use]
mod json_str;
mod cast;
mod full_judge_report;
mod subm_raw;
mod username;

pub use cast::{CastElapse, CastMemory};
pub use datetime::DateTime;
pub use email::EmailAddress;
pub use full_judge_report::FullJudgeReport;
pub use gender::Gender;
pub use json_str::JsonStr;
pub use subm_raw::SubmRaw;
pub use username::Username;
