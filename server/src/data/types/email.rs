//! 自定义用户名的类型，实现内容正确性检验

use super::*;
use std::{
    borrow::{Borrow, BorrowMut},
    str::FromStr,
};

/// 邮箱类型，在创建时会进行内容检查，确保没有不合法字符
#[derive(
    Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, SqlType, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct EmailAddress(email_address::EmailAddress);

impl serde_ts_typing::TsType for EmailAddress {
    fn register_context(_: &mut serde_ts_typing::Context) {
        // nothing
    }

    fn type_def() -> serde_ts_typing::TypeExpr {
        serde_ts_typing::TypeExpr::String
    }
}

impl EmailAddress {
    /// 检查字符串内容并新建一个用户名
    pub fn new(value: impl AsRef<str>) -> Result<Self, email_address::Error> {
        email_address::EmailAddress::from_str(value.as_ref()).map(Self)
    }
}
impl Borrow<email_address::EmailAddress> for EmailAddress {
    fn borrow(&self) -> &email_address::EmailAddress {
        &self.0
    }
}
impl BorrowMut<email_address::EmailAddress> for EmailAddress {
    fn borrow_mut(&mut self) -> &mut email_address::EmailAddress {
        &mut self.0
    }
}
impl ToString for EmailAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
impl From<email_address::EmailAddress> for EmailAddress {
    fn from(value: email_address::EmailAddress) -> Self {
        Self(value)
    }
}

mod mysql {
    use super::*;

    impl<DB> serialize::ToSql<Text, DB> for EmailAddress
    where
        DB: backend::Backend,
        str: serialize::ToSql<Text, DB>,
    {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
            self.0.as_str().to_sql(out)
        }
    }

    impl<DB> deserialize::FromSql<Text, DB> for EmailAddress
    where
        DB: backend::Backend,
        String: deserialize::FromSql<Text, DB>,
    {
        fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
            let r = String::from_sql(bytes)?;
            Ok(EmailAddress::new(r)?)
        }
    }
}
