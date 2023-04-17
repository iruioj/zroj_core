use actix_web::{Result, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub host: String,
    pub port: u16,
    pub userdata_database_url: Option<String>,
    pub problem_base_dir: String,
    pub problem_statement: String,
    pub problem_data_dir: String,
    pub judge_count: usize,
    pub user_data_path: String,
    pub app_config_path: String,
}
impl CoreConfig {
    /* pub fn set <T> (key: String, value : T) {
        TODO, but later;
    }*/
    pub fn load() -> Result<Self> {
        let s = std::fs::read_to_string("./core.yml").map_err(|_| {
            error::ErrorInternalServerError("Fail to read core config from path: ./core.yml")
        })?;
        Ok(serde_json::from_str::<Self>(&s)
            .map_err(|_| error::ErrorInternalServerError("Fail to parse file content as data"))?)
    }
    pub fn new() -> Self {
        if let Ok(o) = Self::load() {
            return o;
        }
        Self {
            host: "127.0.0.1".to_string(),
            port: 80,
            userdata_database_url: None,
            problem_base_dir: "/var/problems/{}/".to_string(),
            problem_statement: "stmt.json".to_string(),
            problem_data_dir: "data/".to_string(),
            judge_count: 8usize,
            user_data_path: "./userinfo.json".to_string(),
            app_config_path: "./appconfig.json".to_string(),
        }
    }
}

