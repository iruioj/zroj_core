//! A wrapper type of [`email_address::EmailAddress`].
use super::*;
use std::{
    borrow::{Borrow, BorrowMut},
    str::FromStr,
};

/// EmailAddress internally use [`email_address::EmailAddress`], which is a
/// new-type struct of [`String`]. It check the sanity when calling [`EmailAddress::new`].
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
    /// Check sanity and create it
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
impl std::fmt::Display for EmailAddress{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
