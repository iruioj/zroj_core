//! imp struct for different database queries
use crate::{
    config::core::CoreConfig,
    schema::{users, NewUser, User},
};
use actix_web::{error::ErrorInternalServerError, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Manager {
    fn new(config: &CoreConfig) -> Self;
    async fn query_by_username(&self, username: &String) -> Result<Option<User>>;
    async fn query_by_userid(&self, userid: i32) -> Result<Option<User>>;
    async fn insert(
        &self,
        username: &String,
        password_hash: &String,
        email: &String,
    ) -> Result<User>;
}

pub mod database {
    use crate::data::user::*;
    use async_trait::async_trait;
    use diesel::{
        self,
        mysql::MysqlConnection,
        prelude::*,
        r2d2::{ConnectionManager, Pool, PooledConnection},
    };
    type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
    type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;
    use crate::data::user::Manager;
    use actix_web::error::{self, Result};

    pub struct Database(MysqlPool);

    impl Database {
        async fn get_conn(&self) -> Result<MysqlPooledConnection> {
            self.0.get().map_err(|e| {
                error::ErrorInternalServerError(format!("Pool connection error: {}", e.to_string()))
            })
        }
    }
    #[async_trait]
    impl Manager for Database {
        fn new(config: &CoreConfig) -> Self {
            Self {
                0: Pool::builder()
                    .max_size(15)
                    .build(ConnectionManager::<MysqlConnection>::new(
                        config
                            .userdata_database_url
                            .clone()
                            .expect("missing user data database url"),
                    ))
                    .expect("fail to establish database connection pool"),
            }
        }
        async fn query_by_username(&self, username: &String) -> Result<Option<User>> {
            let mut conn = self.get_conn().await?;
            let result = users::table
                .filter(users::username.eq(username))
                .first::<User>(&mut conn);
            match result {
                Ok(user) => Ok(Some(user)),
                Err(e) => match e {
                    diesel::result::Error::NotFound => Ok(None),
                    _ => Err(ErrorInternalServerError(format!(
                        "Database error: {}",
                        e.to_string()
                    ))),
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
                    _ => Err(ErrorInternalServerError(format!(
                        "Database error: {}",
                        e.to_string()
                    ))),
                },
            }
        }
        async fn insert(
            &self,
            username: &String,
            password_hash: &String,
            email: &String,
        ) -> Result<User> {
            let mut conn = self.get_conn().await?;
            conn.transaction(|conn| {
                let new_user = NewUser {
                    username: username,
                    password_hash: password_hash,
                    email: email,
                };
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .execute(conn)?;
                users::table.order(users::id.desc()).first::<User>(conn)
            })
            .map_err(|e| ErrorInternalServerError(format!("Database Error: {}", e.to_string())))
        }
    }
}

pub mod hashmap {
    use std::sync::RwLock;

    use crate::data::user::Manager;
    use crate::{auth::UserID, config::core::CoreConfig, data::user::*};
    use actix_web::error::{self, Result};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::from_str;

    #[derive(Serialize, Deserialize)]
    struct Data(
        std::collections::HashMap<String, UserID>,
        std::collections::HashMap<UserID, User>,
        UserID,
    );

    #[derive(Serialize, Deserialize)]
    pub struct HashMap {
        data: RwLock<Data>,
        path: String,
    }

    impl HashMap {
        fn load(path: &String) -> std::result::Result<Data, ()> {
            let s = std::fs::read_to_string(path)
                .map_err(|_| eprintln!("Fail to read from path: {}", path))?;
            Ok(from_str::<Data>(&s)
                .map_err(|_| eprintln!("Fail to parse file content as user data"))?)
        }
        /// save data to json file, must be saved or panic!!!
        fn save(&self) {
            let guard = self.data.read().expect("Fail to fetch guard when saving");
            let s = serde_json::to_string::<Data>(&guard).expect("Fail to parse user data as json");
            std::fs::write(&self.path, s)
                .expect(&format!("Fail to write user data to path: {}", self.path));
        }
    }
    #[async_trait]
    impl Manager for HashMap {
        fn new(config: &CoreConfig) -> Self {
            let r = Self::load(&config.user_data_path).unwrap_or(Data {
                0: std::collections::HashMap::new(),
                1: std::collections::HashMap::new(),
                2: 0,
            });
            Self {
                data: RwLock::new(r),
                path: config.user_data_path.clone(),
            }
        }
        async fn query_by_username(&self, username: &String) -> Result<Option<User>> {
            let guard = self
                .data
                .read()
                .map_err(|_| error::ErrorInternalServerError("Fail to get read lock"))?;
            if let Some(uid) = guard.0.get(username) {
                match guard.1.get(uid) {
                    Some(v) => Ok(Some(v.clone())),
                    None => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        async fn query_by_userid(&self, uid: UserID) -> Result<Option<User>> {
            let guard = self
                .data
                .read()
                .map_err(|_| error::ErrorInternalServerError("Fail to get read lock"))?;
            match guard.1.get(&uid) {
                Some(v) => Ok(Some(v.clone())),
                None => Ok(None),
            }
        }
        async fn insert(
            &self,
            username: &String,
            password_hash: &String,
            email: &String,
        ) -> Result<User> {
            let mut guard = self
                .data
                .write()
                .map_err(|_| error::ErrorInternalServerError("Fail to get write lock"))?;
            let new_user = User {
                id: guard.2,
                username: username.clone(),
                password_hash: password_hash.clone(),
                email: email.clone(),
            };
            guard.2 = guard.2 + 1;
            guard.0.insert(new_user.username.clone(), new_user.id);
            if let Some(_) = guard.1.insert(new_user.id, new_user.clone()) {
                panic!("Error, duplicate userid is not expected");
            }
            drop(guard);
            self.save();
            Ok(new_user)
        }
    }
}
