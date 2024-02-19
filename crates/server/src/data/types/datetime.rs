//! A wrapper type of [`chrono::DateTime`].

use super::*;
use std::fmt::Display;

/// 时间表示（时间和 timestamp）
#[derive(
    Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, SqlType, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct DateTime(
    #[serde(with = "chrono::serde::ts_milliseconds")] chrono::DateTime<chrono::Utc>,
);

impl DateTime {
    pub fn now() -> Self {
        let t = chrono::Utc::now();
        Self(t)
    }
    pub fn now_with_offset_seconds(sec: i64) -> Self {
        Self::try_from(chrono::Utc::now().timestamp_millis() + sec * 1000).unwrap()
    }
    pub fn to_i64(&self) -> i64 {
        self.0.timestamp_millis()
    }
}
impl TryFrom<String> for DateTime {
    type Error = chrono::format::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: chrono::DateTime<chrono::Utc> = value.parse()?;
        Ok(Self(v))
    }
}
impl TryFrom<i64> for DateTime {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use chrono::TimeZone;
        let t = chrono::Utc.timestamp_millis_opt(value).unwrap();
        Ok(Self(t))
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

mod mysql {
    use super::*;

    impl serialize::ToSql<BigInt, diesel::mysql::Mysql> for DateTime
    where
        i64: serialize::ToSql<BigInt, diesel::mysql::Mysql>,
    {
        fn to_sql<'b>(
            &'b self,
            out: &mut serialize::Output<'b, '_, diesel::mysql::Mysql>,
        ) -> serialize::Result {
            let v = self.to_i64();
            <i64 as serialize::ToSql<BigInt, diesel::mysql::Mysql>>::to_sql(&v, &mut out.reborrow())
        }
    }

    impl<DB> deserialize::FromSql<BigInt, DB> for DateTime
    where
        DB: backend::Backend,
        i64: deserialize::FromSql<BigInt, DB>,
    {
        fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
            let r = i64::from_sql(bytes)?;

            Ok(Self::try_from(r).unwrap())
        }
    }
}
