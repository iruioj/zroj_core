use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{schema::{users, NewUser, User}, config::core::CoreConfig};
use actix_web::App;
use serde::{Serialize, Deserialize};
use serde_json::from_str;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    
}

pub struct AppConfigManager {
    data: RwLock<AppConfig>,
    path: String,
}
impl AppConfigManager {
    fn load(path: &String) -> std::result::Result<AppConfig, ()> {
        let s = std::fs::read_to_string(path)
            .map_err(|_| eprintln!("Fail to read from path: {}", path))?;
        Ok(from_str::<AppConfig>(&s)
            .map_err(|_| eprintln!("Fail to parse file content as user data"))?)
    }
    /// save data to json file, must be saved or panic!!!
    fn save(&self) {
        let guard = self.data.read().expect("Fail to fetch guard when saving");
        let s = serde_json::to_string::<AppConfig>(&guard).expect("Fail to parse user data as json");
        std::fs::write(&self.path, s)
            .expect(&format!("Fail to write user data to path: {}", self.path));
    }
    fn new(config: CoreConfig) -> Self {
        let data = Self::load(&config.app_config_path)
            .unwrap_or(AppConfig {
            });
        Self {
            data: RwLock::new(data),
            path: config.app_config_path.clone(),
        }
    }
    fn read(&self) -> actix_web::Result <RwLockReadGuard<AppConfig>> {
        self.data.read()
            .map_err(|e| actix_web::error::ErrorInternalServerError("Fail to get read lock"))
    }
    fn write(&self) -> actix_web::Result <RwLockWriteGuard<AppConfig>> {
        self.data.write()
            .map_err(|e| actix_web::error::ErrorInternalServerError("Fail to get write lock"))
    }
}




