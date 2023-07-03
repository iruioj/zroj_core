//! 自定义用户名的类型，实现内容正确性检验

use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_ts_typing::SerdeJsonWithType;
#[cfg(feature = "mysql")]
use super::*;

/// 用户名类型，在创建时会进行内容检查，确保没有不合法字符
#[derive(Debug, Serialize, Clone, Hash, PartialEq, Eq, SerdeJsonWithType)]
#[cfg_attr(feature = "mysql", derive(SqlType, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "mysql", diesel(sql_type = Text))]
pub struct Username(String);

#[derive(Debug)]
pub enum Error {
    TooLong,
    TooShort,
    InvalidChar(char)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::TooLong => write!(f, "username too long (> 20)"),
            Error::TooShort => write!(f, "username too short (< 6)"),
            Error::InvalidChar(c) => write!(f, "username contains invalid char '{c}'"),
        }
    }
}

impl std::error::Error for Error {}

impl Username {
    /// 检查字符串内容并新建一个用户名
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        let value = value.as_ref().to_string();
        if value.len() < 6 {
            Err(Error::TooShort)
        } else if value.len() > 20 {
            Err(Error::TooLong)
        } else {
            match value.chars().find(|c| !(c.is_alphanumeric() || *c == '_')) {
                Some(c) => Err(Error::InvalidChar(c)),
                None => Ok(Self(value)),
            }
        }
    }
}
impl AsRef<String> for Username {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
impl ToString for Username {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
// for cookie builder
impl From<Username> for std::borrow::Cow<'_, str> {
    fn from(value: Username) -> Self {
        value.0.into()
    }
}

// 在反序列化时检查内容合法性
impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct UsernameVisitor;

        impl Visitor<'_> for UsernameVisitor {
            type Value = Username;

            fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.write_str("a string of letters, numbers and '_', lengthing between [6, 20]")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Username::new(s).map_err(|e| Error::invalid_value(
                    Unexpected::Str(s),
                    &e.to_string().as_str(),
                ))
            }
        }

        deserializer.deserialize_str(UsernameVisitor)
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use super::*;

    impl<DB> serialize::ToSql<Text, DB> for Username
    where
        DB: backend::Backend,
        String: serialize::ToSql<Text, DB>,
    {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
            self.0.to_sql(out)
        }
    }

    impl<DB> deserialize::FromSql<Text, DB> for Username
    where
        DB: backend::Backend,
        String: deserialize::FromSql<Text, DB>,
    {
        fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
            let r = String::from_sql(bytes)?;
            Ok(Username::new(r)?)
        }
    }
}
