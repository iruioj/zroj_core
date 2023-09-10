//! some utils for mysql database

pub mod schema;
pub mod schema_model;
// pub use schema::users;

use crate::data::error::DataError;
use diesel::{
    self,
    mysql::MysqlConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

#[derive(Debug, Clone)]
pub struct MysqlConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
}

/// A MysqlDb is simply a connection pool
pub struct MysqlDb(MysqlPool);

/// 数据库存储
impl MysqlDb {
    pub fn new(cfg: &MysqlConfig) -> Self {
        let MysqlConfig {
            user,
            password,
            host,
            port,
            dbname,
        } = cfg;
        let url = format!("mysql://{user}:{password}@{host}:{port}/{dbname}");
        Self(
            Pool::builder()
                .max_size(15)
                .build(ConnectionManager::<MysqlConnection>::new(url))
                .expect("fail to establish database connection pool"),
        )
    }
    /// remove original tables and create new ones
    pub fn setup_new(cfg: &MysqlConfig) -> Result<Self, DataError> {
        setup_database(cfg, SetupDatabaseFlag::ForceNew)?;
        tracing::debug!(?cfg, "create database");

        let r = Self::new(cfg);
        r.transaction(|conn| {
            // for cmd in include_str!("./drop_tables.sql").split(";").map(str::trim) {
            //     if !cmd.is_empty() {
            //         tracing::debug!("executing: {}", cmd);
            //         diesel::sql_query(cmd).execute(conn)?;
            //     }
            // }

            // for cmd in include_str!("./create_tables.sql")
            //     .split(";")
            //     .map(str::trim)
            // {
            //     if !cmd.is_empty() {
            //         tracing::debug!("executing: {}", cmd);
            //         diesel::sql_query(cmd).execute(conn)?;
            //     }
            // }
            Ok(())
        })?;

        Ok(r)
    }
    /// see [`diesel::connection::Connection::transaction`]
    pub fn transaction<T, F>(&self, f: F) -> Result<T, DataError>
    where
        F: FnOnce(&mut MysqlPooledConnection) -> Result<T, DataError>,
    {
        self.0.get()?.transaction(f)
    }
}

pub enum SetupDatabaseFlag {
    CreateIfNotExist,
    ForceNew,
}

pub fn setup_database(cfg: &MysqlConfig, flag: SetupDatabaseFlag) -> Result<(), DataError> {
    let MysqlConfig {
        user,
        password,
        host,
        port,
        ..
    } = cfg;
    let url = format!("mysql://{user}:{password}@{host}:{port}/information_schema");
    let mut conn = MysqlConnection::establish(&url).unwrap();
    tracing::debug!(?url, "establish connection");
    conn.transaction(|conn| {
        if matches!(flag, SetupDatabaseFlag::CreateIfNotExist) {
            diesel::sql_query(format!(
                "CREATE DATABASE IF NOT EXISTS {} CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_520_ci",
                cfg.dbname
            ))
            .execute(conn)?;
        } else if matches!(flag, SetupDatabaseFlag::ForceNew) {
            diesel::sql_query(format!("DROP DATABASE IF EXISTS {}", cfg.dbname)).execute(conn)?;
            diesel::sql_query(format!(
                "CREATE DATABASE {} CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_520_ci",
                cfg.dbname
            ))
            .execute(conn)?;
        }
        Ok(())
    })
}

// mysql only
sql_function! { fn last_insert_id() -> Unsigned<BigInt>; }