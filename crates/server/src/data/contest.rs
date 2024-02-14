use crate::CtstID;

use super::{
    error::DataError,
    mysql::{schema::contests, schema_model::Contest, MysqlDb},
    types::*,
};
use diesel::*;
use serde::Serialize;
use serde_ts_typing::TsType;

pub struct CtstDB(MysqlDb);

impl CtstDB {
    pub fn new(mysqldb: &MysqlDb) -> Self {
        Self(mysqldb.clone())
    }
    pub fn get_metas(
        &self,
        max_count: u8,
        offset: usize,
        pattern: Option<String>,
    ) -> Result<Vec<ContestMeta>, DataError> {
        self.0.transaction(|conn| {
            Ok(contests::table
                .filter(
                    contests::title.like(
                        pattern
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("%".into()),
                    ),
                )
                .offset(offset as i64)
                .limit(max_count as i64)
                .load::<Contest>(conn)?
                .into_iter()
                .map(|c: Contest| ContestMeta {
                    id: c.id,
                    title: c.title,
                    start_time: c.start_time.to_i64(),
                    end_time: c.end_time.to_i64(),
                    duration: c.duration,
                })
                .collect())
        })
    }
}

#[derive(TsType, Serialize)]
pub struct ContestMeta {
    pub id: CtstID,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration: CastElapse,
}
