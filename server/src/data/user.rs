//! imp struct for different database queries
use super::error::Result;
use crate::data::schema::User;
use async_trait::async_trait;
use std::sync::Arc;

pub type AManager = dyn Manager + Sync + Send;

#[async_trait]
pub trait Manager {
    async fn query_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn query_by_userid(&self, userid: i32) -> Result<Option<User>>;
    async fn insert(&self, username: &str, password_hash: &str, email: &str) -> Result<User>;
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
        pub fn new(url: &String) -> Self {
            Self {
                0: Pool::builder()
                    .max_size(15)
                    .build(ConnectionManager::<MysqlConnection>::new(url.clone()))
                    .expect("fail to establish database connection pool"),
            }
        }
        async fn get_conn(&self) -> Result<MysqlPooledConnection> {
            Ok(self.0.get()?)
            //.map_err(|e| {
            //error::ErrorInternalServerError(format!("Pool connection error: {}", e.to_string()))
            // })
        }
    }
    #[async_trait]
    impl Manager for DbManager {
        async fn query_by_username(&self, username: &str) -> Result<Option<User>> {
            let mut conn = self.get_conn().await?;
            let result = users::table
                .filter(users::username.eq(username))
                .first::<User>(&mut conn);
            match result {
                Ok(user) => Ok(Some(user)),
                Err(e) => match e {
                    diesel::result::Error::NotFound => Ok(None),
                    ee => Err(Error::DbError(ee)),
                },
            }
        }
        async fn query_by_userid(&self, userid: i32) -> Result<Option<User>> {
            let mut conn = self.get_conn().await?;
            let result = users::table.filter(users::id.eq(userid)).first(&mut conn);
            match result {
                Ok(user) => Ok(Some(user)),
                Err(e) => match e {
                    diesel::result::Error::NotFound => Ok(None),
                    ee => Err(Error::DbError(ee)),
                },
            }
        }
        async fn insert(&self, username: &str, password_hash: &str, email: &str) -> Result<User> {
            let mut conn = self.get_conn().await?;
            conn.transaction(|conn| {
                let new_user = NewUser {
                    username,
                    password_hash,
                    email,
                };
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .execute(conn)?;
                users::table.order(users::id.desc()).first::<User>(conn)
            })
            .map_err(|e| Error::DbError(e))
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}

pub use hashmap::FsManager;
mod hashmap {
    use crate::{auth::UserID, data::user::*};
    use serde::{Deserialize, Serialize};
    use serde_json::from_str;
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
                path: path.clone(),
            }
        }
        fn load(path: &PathBuf) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("Fail to read from path: {}", path.display()))?;
            Ok(from_str::<Data>(&s)
                .map_err(|_| eprintln!("Fail to parse file content as user data"))?)
        }
        /// save data to json file, must be saved or panic!!!
        fn save(&self) {
            let guard = self.data.read().expect("Fail to fetch guard when saving");
            let s = serde_json::to_string::<Data>(&guard).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s).expect(&format!(
                "Fail to write user data to path: {}",
                self.path.display()
            ));
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
            if uid < 0 || uid as usize >= guard.1.len() {
                Ok(None)
            } else {
                Ok(Some(guard.1[uid as usize].clone()))
            }
        }
        async fn insert(&self, username: &str, password_hash: &str, email: &str) -> Result<User> {
            let mut guard = self.data.write()?;
            let new_user = User {
                id: guard.1.len() as UserID,
                username: username.to_string(),
                password_hash: password_hash.to_string(),
                email: email.to_string(),
            };
            guard.0.insert(new_user.username.clone(), new_user.id);
            guard.1.push(new_user.clone());
            drop(guard);
            self.save();
            Ok(new_user)
        }
        /// consume self and return its Arc.
        fn to_amanager(self) -> Arc<AManager> {
            Arc::new(self)
        }
    }
}
