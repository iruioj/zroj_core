//! 用户数据库

use super::error::{Error, Result};
use crate::data::schema::Gender;
use crate::{app::user::UserUpdateInfo, data::schema::User};
use crate::{GroupID, Override, UserID};
use async_trait::async_trait;
use std::sync::Arc;
pub type AManager = dyn Manager + Sync + Send;
use serde::{Deserialize, Serialize};

// Result<Option<...>> pattern: Err 表示出错， None 表示未查到，Some 表示查到的值
#[async_trait]
pub trait Manager {
    async fn query_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>>;
    async fn new_user(&self, username: &str, password_hash: &str, email: &str) -> Result<User>;
    async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<()>;
    fn to_amanager(self) -> Arc<AManager>;
}

#[cfg(feature = "mysql")]
pub use database::DbManager;

#[cfg(feature = "mysql")]
mod database {
    use crate::data::error::Error;
    use crate::data::schema::{users, NewUser};
    use crate::data::user::*;
    use diesel::{
        self,
        mysql::MysqlConnection,
        prelude::*,
        r2d2::{ConnectionManager, Pool, PooledConnection},
    };
    type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
    type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;
    use crate::data::user::Manager;

    pub struct DbManager(MysqlPool);
    
    /// 数据库存储
    impl DbManager {
        pub fn new(url: impl AsRef<str>) -> Self {
            Self(Pool::builder()
                    .max_size(15)
                    .build(ConnectionManager::<MysqlConnection>::new(url.as_ref()))
                    .expect("fail to establish database connection pool"))
        }
        async fn get_conn(&self) -> Result<MysqlPooledConnection> {
            Ok(self.0.get()?)
            //.map_err(|e| {
            //error::ErrorInternalServerError(format!("Pool connection error: {}", e.to_string()))
            // })
        }
        async fn _query_by_username(
            &self,
            conn: &mut MysqlPooledConnection,
            username: &str,
        ) -> Result<Option<User>> {
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
        ) -> Result<Option<User>> {
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
    #[async_trait]
    impl Manager for DbManager {
        async fn query_by_username(&self, username: &str) -> Result<Option<User>> {
            self._query_by_username(&mut self.get_conn().await?, username)
                .await
        }
        async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>> {
            self._query_by_userid(&mut self.get_conn().await?, uid)
                .await
        }
        async fn new_user(&self, username: &str, password_hash: &str, email: &str) -> Result<User> {
            let mut conn = self.get_conn().await?;
            conn.transaction(|conn| {
                let new_user = NewUser {
                    username,
                    password_hash,
                    email,
                    register_time: chrono::Local::now().to_string(),
                    gender: Gender::Private as u32,
                    groups: serde_json::to_string(&Vec::<GroupID>::new()).unwrap(),
                };
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .execute(conn)?;
                users::table.order(users::id.desc()).first::<User>(conn)
            })
            .map_err(Error::DbError)
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
        async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<()> {
            let mut conn = self.get_conn().await?;
            let mut user =
                self._query_by_userid(&mut conn, uid)
                    .await?
                    .ok_or(Error::InvalidArgument(format!(
                        "User {} does not exist",
                        uid
                    )))?;
            info.over(&mut user);
            diesel::update(users::table.filter(users::id.eq(uid)))
                .set(user)
                .execute(&mut conn)?;
            Ok(())
        }
    }
}

pub use default::FsManager;

mod default {
    //! 默认实现方式（无 sql 依赖）

    use super::*;
    use std::sync::RwLock;
    use std::{collections::HashMap, path::PathBuf};

    #[derive(Serialize, Deserialize)]
    struct Data(HashMap<String, UserID>, Vec<User>);

    /// 文件系统存储信息
    #[derive(Serialize, Deserialize)]
    pub struct FsManager {
        data: RwLock<Data>,
        path: PathBuf,
    }

    impl FsManager {
        pub fn new(path: PathBuf) -> Self {
            let r = Self::load(&path).unwrap_or(Data(HashMap::new(), Vec::new()));
            Self {
                data: RwLock::new(r),
                path,
            }
        }
        fn load(path: &PathBuf) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("warning: fail to read from path: {}", path.display()))?;
            serde_json::from_str::<Data>(&s)
                .map_err(|_| eprintln!("warning: fail to parse file content as user data"))
        }
        /// save data to json file, must be saved or panic!!!
        fn save(&self) {
            let guard = self.data.read().expect("Fail to fetch guard when saving");
            let s = serde_json::to_string::<Data>(&guard).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s).unwrap_or_else(|_| panic!("Fail to write user data to path: {}",
                self.path.display()));
        }
    }
    #[async_trait]
    impl super::Manager for FsManager {
        async fn query_by_username(&self, username: &str) -> Result<Option<User>> {
            let guard = self.data.read()?;
            // .map_err(|_| error::ErrorInternalServerError("Fail to get read lock"))?;
            if let Some(uid) = guard.0.get(username) {
                Ok(Some(guard.1[*uid as usize].clone()))
            } else {
                Ok(None)
            }
        }
        async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>> {
            let guard = self.data.read()?;
            if uid as usize >= guard.1.len() {
                Ok(None)
            } else {
                Ok(Some(guard.1[uid as usize].clone()))
            }
        }
        async fn new_user(&self, username: &str, password_hash: &str, email: &str) -> Result<User> {
            let mut guard = self.data.write()?;
            let new_user = User {
                id: guard.1.len() as UserID,
                username: username.to_string(),
                password_hash: password_hash.to_string(),
                email: email.to_string(),
                motto: String::new(),
                name: String::new(),
                register_time: chrono::Local::now().to_string(),
                gender: Gender::Private as u32,
                groups: serde_json::to_string(&Vec::<GroupID>::new()).unwrap(),
            };
            guard.0.insert(new_user.username.clone(), new_user.id);
            guard.1.push(new_user.clone());
            drop(guard);
            self.save();
            Ok(new_user)
        }
        async fn update(&self, uid: UserID, info: UserUpdateInfo) -> Result<()> {
            let mut guard = self.data.write()?;
            if uid as usize >= guard.1.len() {
                return Err(Error::InvalidArgument(format!(
                    "userid = {} violates range: [{}, {})",
                    uid,
                    0,
                    guard.1.len()
                )));
            }
            let value = &mut guard.1[uid as usize];
            info.over(value);
            drop(guard);
            self.save();
            Ok(())
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
