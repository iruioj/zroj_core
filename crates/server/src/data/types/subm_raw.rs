use super::*;
use judger::SourceFile;
use serde_ts_typing::TsType;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

/// Raw content of user submission is stored on file system.
/// This struct provides entries of files in the submission.
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

impl Deref for SubmRaw {
    type Target = BTreeMap<String, SourceFile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SubmRaw {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl_serde_json_sql! {SubmRaw}
