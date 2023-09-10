use judger::JudgeReport;
use super::*;
use serde_ts_typing::TsType;

#[derive(Serialize, Deserialize, TsType, Debug, Clone)]
#[derive(SqlType, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct FullJudgeReport {
    pub pre: Option<JudgeReport>,
    pub data: Option<JudgeReport>,
    pub extra: Option<JudgeReport>,
}

impl FullJudgeReport {
    pub(crate) fn update(&mut self, other: FullJudgeReport) {
        if let Some(pre) = other.pre {
            self.pre.replace(pre);
        }
        if let Some(data) = other.data {
            self.data.replace(data);
        }
        if let Some(extra) = other.extra {
            self.extra.replace(extra);
        }
    }
}

impl_serde_json_sql!{FullJudgeReport}