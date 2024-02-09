use super::*;
use judger::{JudgeReport, Status};
use problem::{Elapse, Memory};
use serde_ts_typing::TsType;

#[derive(Serialize, Deserialize, TsType, Debug, Clone, SqlType, FromSqlRow, AsExpression)]
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
    pub(crate) fn max_memory(&self) -> Memory {
        let get =
            |d: &Option<judger::JudgeReport>| d.as_ref().map(|d| d.meta.memory).unwrap_or_default();
        let m_data = get(&self.data);
        let m_pre = get(&self.pre);
        let m_extra = get(&self.extra);
        m_data.max(m_pre.max(m_extra))
    }
    pub(crate) fn max_time(&self) -> Elapse {
        let get =
            |d: &Option<judger::JudgeReport>| d.as_ref().map(|d| d.meta.time).unwrap_or_default();
        let m_data = get(&self.data);
        let m_pre = get(&self.pre);
        let m_extra = get(&self.extra);
        m_data.max(m_pre.max(m_extra))
    }
    pub(crate) fn status(&self) -> Option<Status> {
        self.data.as_ref().map(|d| d.meta.status.clone())
    }
}

impl_serde_json_sql! {FullJudgeReport}
