//! Define basic data types for SQL serialization.
//!
//! It is recommended to create wrapper types as a restriction of data
//! (e.g. [`Username`], [`EmailAddress`]), so as to prevent attacks.

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
