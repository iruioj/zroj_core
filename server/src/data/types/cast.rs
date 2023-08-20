use super::*;
use problem::{Elapse, Memory};
use serde_ts_typing::TsType;

macro_rules! define_cast {
    ($name:ident, $inner:ty, $primitive:ty, $sql_type:ty) => {
/// 使用 TryFrom 和 TryInto 转换为基础类型
#[derive(std::fmt::Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mysql", derive(SqlType, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "mysql", diesel(sql_type = $sql_type))]
pub struct $name(pub $inner);

impl TsType for $name {
    fn register_context(c: &mut serde_ts_typing::Context) {
        <$inner as TsType>::register_context(c)
    }

    fn type_def() -> serde_ts_typing::TypeExpr {
        <$inner as TsType>::type_def()
    }
}

#[cfg(feature = "mysql")]
const _: () = {
    use super::*;
    use diesel::mysql::Mysql;
    impl serialize::ToSql<$sql_type, Mysql> for $name {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Mysql>) -> serialize::Result {
            <$primitive as serialize::ToSql<$sql_type, Mysql>>::to_sql(
                &TryInto::<$primitive>::try_into(self.0)?,
                &mut out.reborrow(),
            )
        }
    }

    impl deserialize::FromSql<$sql_type, Mysql> for $name {
        fn from_sql(
            bytes: <Mysql as diesel::backend::Backend>::RawValue<'_>,
        ) -> deserialize::Result<Self> {
            let s = <$primitive as deserialize::FromSql<$sql_type, Mysql>>::from_sql(bytes)?;
            Ok($name(TryInto::<$inner>::try_into(s)?))
        }
    }
};

    };
}

define_cast!{CastElapse, Elapse, u64, Unsigned<BigInt>}
define_cast!{CastMemory, Memory, u64, Unsigned<BigInt>}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    enum TestEnum {
        A,
        B,
        C(u8),
    }
}
