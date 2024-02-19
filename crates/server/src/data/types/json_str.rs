//! Wrapper type builder for any serializable data types.
//!
//! 对于自定义的类型，可以使用 [`impl_serde_json_sql`] macro 来实现 ToSql, FromSql，
//! 对于非自定义的类型可以使用 [`JsonStr`] Wrapper 实现
use std::fmt::Debug;

use super::*;
use serde_ts_typing::TsType;

/// 使用 serde_json 转化为字符串存储在数据库中
#[derive(Debug, Clone, Serialize, SqlType, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
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

#[macro_export]
macro_rules! impl_serde_json_sql {
    ($type:ty) => {
        const _: () = {
            use diesel::mysql::Mysql;
            impl serialize::ToSql<Text, Mysql> for $type {
                fn to_sql<'b>(
                    &'b self,
                    out: &mut serialize::Output<'b, '_, Mysql>,
                ) -> serialize::Result {
                    let v = serde_json::to_string(&self).expect("data should be serialize to json");
                    <String as serialize::ToSql<Text, Mysql>>::to_sql(&v, &mut out.reborrow())
                }
            }

            impl deserialize::FromSql<Text, Mysql> for $type {
                fn from_sql(
                    bytes: <Mysql as diesel::backend::Backend>::RawValue<'_>,
                ) -> deserialize::Result<Self> {
                    let s = <String as deserialize::FromSql<Text, Mysql>>::from_sql(bytes)?;
                    Ok(serde_json::from_str(&s)?)
                }
            }
        };
    };
}

const _: () = {
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
};

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
