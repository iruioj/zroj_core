//! 使用 serde_json 转化为字符串存储
use std::fmt::Debug;

use super::*;
use serde_ts_typing::TsType;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "mysql", derive(SqlType, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "mysql", diesel(sql_type = Text))]
pub struct JsonStr<T: Sized + Serialize + for<'de> Deserialize<'de> + Debug + 'static>(pub T);

impl<'de, T> Deserialize<'de> for JsonStr<T>
where
    T: Serialize + for<'d> Deserialize<'d> + Debug + 'static,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|r| JsonStr(r))
    }
}

impl<T> TsType for JsonStr<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + 'static + TsType,
{
    fn register_context(c: &mut serde_ts_typing::Context) {
        T::register_context(c)
    }

    fn type_def() -> serde_ts_typing::TypeExpr {
        T::type_def()
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use super::*;
    use diesel::mysql::Mysql;
    impl<T> serialize::ToSql<Text, Mysql> for JsonStr<T>
    where
        Mysql: backend::Backend,
        T: Serialize + for<'de> Deserialize<'de> + Debug + 'static,
    {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Mysql>) -> serialize::Result {
            let v = serde_json::to_string(&self.0).expect("data should be serialize to json");
            <String as serialize::ToSql<Text, Mysql>>::to_sql(&v, &mut out.reborrow())
        }
    }

    impl<T> deserialize::FromSql<Text, Mysql> for JsonStr<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + 'static,
    {
        fn from_sql(
            bytes: <Mysql as diesel::backend::Backend>::RawValue<'_>,
        ) -> deserialize::Result<Self> {
            let s = <String as deserialize::FromSql<Text, Mysql>>::from_sql(bytes)?;
            Ok(JsonStr(serde_json::from_str::<T>(&s)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    enum TestEnum {
        A,
        B,
        C(u8),
    }

    #[test]
    fn test_serde() {
        let a = JsonStr(TestEnum::C(8));
        let v = serde_json::to_value(a).unwrap();
        let a2: JsonStr<TestEnum> = serde_json::from_value(v).unwrap();

        dbg!(a2);
    }
}
