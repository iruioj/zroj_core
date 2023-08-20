use super::*;
use serde_ts_typing::TsType;

/// 性别类型
///
/// TODO: 更多的性别
#[derive(Debug, Serialize, Deserialize, Clone, TsType)]
pub enum GenderInner {
    Male,
    Female,
    Others,
    Private,
}

pub type Gender = JsonStr<GenderInner>;