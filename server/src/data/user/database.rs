use crate::data::error::Error;
use crate::data::types::*;
use crate::data::user::{async_trait, Manager, User, UserID, UserUpdateInfo};
use crate::Override;
use diesel::{
    self,
    mysql::MysqlConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

use diesel::{table, Insertable};

// 必须保证和 User 的字段顺序相同， 不然 query 会出问题
table! {
    users (id) {
        /// id should be auto increment
        id -> Unsigned<Integer>,
        username -> Text,
        password_hash -> Text,
        name -> Text,
        email -> Text,
        motto -> Text,
        register_time -> BigInt,
        gender -> Unsigned<Integer>,
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a Username,
    pub password_hash: &'a str,
    pub email: &'a EmailAddress,
    pub register_time: &'a DateTime,
    pub gender: &'a Gender,
}
pub struct DbManager(MysqlPool);

/// 数据库存储
impl DbManager {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self(
            Pool::builder()
                .max_size(15)
                .build(ConnectionManager::<MysqlConnection>::new(url.as_ref()))
                .expect("fail to establish database connection pool"),
        )
    }
    async fn get_conn(&self) -> Result<MysqlPooledConnection, Error> {
        Ok(self.0.get()?)
        //.map_err(|e| {
        //error::ErrorInternalServerError(format!("Pool connection error: {}", e.to_string()))
        // })
    }
    async fn _query_by_username(
        &self,
        conn: &mut MysqlPooledConnection,
        username: &str,
    ) -> Result<Option<User>, Error> {
        let result = users::table
            .filter(users::username.eq(username))
            .first::<User>(conn);
        match result {
            Ok(user) => Ok(Some(user)),
            Err(e) => match e {
                diesel::result::Error::NotFound => Ok(None),
                ee => Err(Error::DbError(ee)),
            },
        }
    }
    async fn _query_by_userid(
        &self,
        conn: &mut MysqlPooledConnection,
        uid: UserID,
    ) -> Result<Option<User>, Error> {
        let result = users::table.filter(users::id.eq(uid)).first(conn);
        match result {
            Ok(user) => Ok(Some(user)),
            Err(e) => match e {
                diesel::result::Error::NotFound => Ok(None),
                ee => Err(Error::DbError(ee)),
            },
        }
    }
}
#[async_trait(?Send)]
impl Manager for DbManager {
    async fn query_by_username(&self, username: &Username) -> Result<Option<User>, Error> {
        self._query_by_username(&mut self.get_conn().await?, username.as_ref())
            .await
    }
    async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>, Error> {
        self._query_by_userid(&mut self.get_conn().await?, uid)
            .await
    }
    async fn new_user(
        &self,
        username: &Username,
        password_hash: &str,
        email: &EmailAddress,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn().await?;
        conn.transaction(|conn| {
            let new_user = NewUser {
                username,
                password_hash,
                email,
                register_time: &DateTime::now(),
                gender: &Gender::Private,
                // groups: serde_json::to_string(&Vec::<GroupID>::new()).unwrap(),
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)?;
            users::table.order(users::id.desc()).first::<User>(conn)
        })
        .map_err(Error::DbError)
    }
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<(), Error> {
        let mut conn = self.get_conn().await?;
        let mut user =
            self._query_by_userid(&mut conn, uid)
                .await?
                .ok_or(Error::InvalidArgument(format!(
                    "user {} does not exist",
                    uid
                )))?;
        info.over(&mut user);
        diesel::update(users::table.filter(users::id.eq(uid)))
            .set(user)
            .execute(&mut conn)?;
        Ok(())
    }
}
