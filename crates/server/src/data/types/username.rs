//! A wrapper type of [`String`].

use super::*;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::fmt::Display;

/// A valid username contains alphabetic letters, numbers, and the underscore `_`.
/// Moreover, its length must lies in `[4, 20]`, and the first character must be alphabetic.
#[derive(
    Debug, Serialize, Clone, Hash, PartialEq, Eq, TsType, SqlType, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct Username(String);

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub enum Error {
    TooLong,
    TooShort,
    InvalidChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::TooLong => write!(f, "username too long (> 20)"),
            Error::TooShort => write!(f, "username too short (< 4)"),
            Error::InvalidChar(c) => write!(f, "username contains invalid char '{c}'"),
        }
    }
}

impl std::error::Error for Error {}

impl Username {
    /// Check sanity and create a new username object
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        // the "root" user should be registered during system initialization,
        // thus is not registerable by user.
        if value.as_ref() == "root" {
            return Ok(Self(value.as_ref().to_string()));
        }

        // avoid the use of regex for performance
        let value = value.as_ref().to_string();
        if value.len() < 4 {
            Err(Error::TooShort)
        } else if value.len() > 20 {
            Err(Error::TooLong)
        } else {
            let first_c = value.chars().next().unwrap();
            if !first_c.is_alphabetic() {
                Err(Error::InvalidChar(first_c))
            } else {
                match value.chars().find(|c| !(c.is_alphanumeric() || *c == '_')) {
                    Some(c) => Err(Error::InvalidChar(c)),
                    None => Ok(Self(value)),
                }
            }
        }
    }
}
impl AsRef<String> for Username {
    fn as_ref(&self) -> &String {
        &self.0
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
                Username::new(s)
                    .map_err(|e| Error::invalid_value(Unexpected::Str(s), &e.to_string().as_str()))
            }
        }

        deserializer.deserialize_str(UsernameVisitor)
    }
}

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
