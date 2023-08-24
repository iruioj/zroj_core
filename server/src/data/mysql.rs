//! some utils for mysql database
use crate::data::error::DataError;
use diesel::{
    self,
    mysql::MysqlConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub struct MysqlDb(MysqlPool);

/// 数据库存储
impl MysqlDb {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self(
            Pool::builder()
                .max_size(15)
                .build(ConnectionManager::<MysqlConnection>::new(url.as_ref()))
                .expect("fail to establish database connection pool"),
        )
    }
    /// see [`diesel::connection::Connection::transaction`]
    pub fn transaction<T, F>(&self, f: F) -> Result<T, DataError>
    where
        F: FnOnce(&mut MysqlPooledConnection) -> Result<T, DataError>,
    {
        self.0.get()?.transaction(f)
    }
}
