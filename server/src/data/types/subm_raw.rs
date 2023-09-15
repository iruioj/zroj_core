use super::*;
use judger::SourceFile;
use serde_ts_typing::TsType;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, TsType, Debug, Clone, SqlType, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct SubmRaw(pub BTreeMap<String, SourceFile>);

const SOURCE_LIMIT: usize = 100 * 1024;

impl SubmRaw {
    pub fn to_display_vec(self) -> Vec<(String, judger::FileType, judger::truncstr::TruncStr)> {
        self.0
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.file_type.clone(),
                    judger::truncstr::TruncStr::new(v.utf8(), SOURCE_LIMIT),
                )
            })
            .collect::<Vec<_>>()
    }
}

impl_serde_json_sql! {SubmRaw}
