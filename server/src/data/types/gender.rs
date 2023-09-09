use crate::impl_serde_json_sql;

use super::*;
use serde_ts_typing::TsType;

/// 性别类型
///
/// TODO: 更多的性别
#[derive(Debug, Serialize, Deserialize, Clone, TsType)]
#[derive(SqlType, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Gender {
    Male,
    Female,
    Others,
    Private,
}

impl_serde_json_sql!{Gender}
