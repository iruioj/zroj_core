use std::fmt::Display;

/// 时间表示（时间和 timestamp）
use super::*;
#[derive(
    Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, SqlType, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct DateTime(
    #[serde(with = "chrono::serde::ts_nanoseconds")] chrono::DateTime<chrono::Utc>,
    i64,
);

impl DateTime {
    pub fn now() -> Self {
        let t = chrono::Utc::now();
        let ts = t.timestamp();
        Self(t, ts)
    }
}
impl TryFrom<String> for DateTime {
    type Error = chrono::format::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: chrono::DateTime<chrono::Utc> = value.parse()?;
        let ts = v.timestamp();
        Ok(Self(v, ts))
    }
}
impl TryFrom<i64> for DateTime {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        use chrono::TimeZone;
        let t = chrono::Utc.timestamp_millis_opt(value).unwrap();
        let ts = t.timestamp();
        Ok(Self(t, ts))
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

mod mysql {
    use super::*;

    impl<DB> serialize::ToSql<BigInt, DB> for DateTime
    where
        DB: backend::Backend,
        i64: serialize::ToSql<BigInt, DB>,
    {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
            self.1.to_sql(out)
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
