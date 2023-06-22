use super::*;

/// 性别类型
///
/// TODO: 更多的性别
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "mysql", derive(SqlType, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "mysql", diesel(sql_type = Unsigned<Integer>))]
pub enum Gender {
    Male = 0,
    Female = 1,
    Others = 2,
    Private = 3,
}


#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    InvalidGender(u32)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::InvalidGender(i) => write!(f, "invalid gender id {i}")
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "mysql")]
mod mysql {
    use super::*;
    impl<DB> serialize::ToSql<Unsigned<Integer>, DB> for Gender
    where
        DB: backend::Backend,
        u32: serialize::ToSql<Unsigned<Integer>, DB>,
    {
        fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
            match self {
                Gender::Male => 0.to_sql(out),
                Gender::Female => 1.to_sql(out),
                Gender::Others => 2.to_sql(out),
                Gender::Private => 3.to_sql(out),
            }
        }
    }

    impl<DB> deserialize::FromSql<Unsigned<Integer>, DB> for Gender
    where
        DB: backend::Backend,
        u32: deserialize::FromSql<Unsigned<Integer>, DB>,
    {
        fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
            let v = u32::from_sql(bytes)?;
            match v {
                0 => Ok(Gender::Male),
                1 => Ok(Gender::Female),
                2 => Ok(Gender::Others),
                3 => Ok(Gender::Private),
                _ => Err(Error::InvalidGender(v))?
            }
        }
    }
}
