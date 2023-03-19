use std::env;
use dotenv::dotenv;
use actix_web::cookie::Key;

fn load_env_default <T>  (key: String, default: T) -> T 
where
    T: std::str::FromStr + std::fmt::Display,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    if let Ok(result) = env::var(key.clone()) {
        match result.parse :: <T>() {
            Ok(result) => result,
            Err(err) => {
                eprintln!("fail to parse key {}, Error: {} set to {}", key, err, default); // print error message
                default
            }
        }
    } else {
        eprintln!("fail to load key {} in .env, set to {}", key, default); // print error message
        default
    }
}
fn load_env_must <T > (key: String) -> T 
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: (core::fmt::Debug),
{
    env::var(key.clone())
        .expect(&format!("key {} must be set in .env", key))
        .parse :: <T> ()
        .expect(&format!("Cannot parse key {}, it must be set correctly", key))
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub secret_key: Key,
    pub problem_base_dir: String,
    pub problem_statement: String,
    pub problem_data_dir: String,
    pub pid_maximum: u32,
}
impl ServerConfig {
    /* pub fn set <T> (key: String, value : T) {
        TODO, but later;
    }*/
}

pub fn load() -> ServerConfig {
    match dotenv() { // try to load .env file
        Ok(_) => println!("Loaded .env file successfully"),
        Err(e) => eprintln!("Failed to load .env file: {}", e),
    };
    ServerConfig {
        host : load_env_default("host".to_string(), "127.0.0.1".to_string()),
        port : load_env_default("port".to_string(), 80),
        database_url: load_env_must("database_url".to_string()),
        problem_base_dir: load_env_default("problem_base_dir".to_string(), "/var/problems/{}/".to_string()),
        problem_statement: load_env_default("problem_statement".to_string(), "stmt.json".to_string()),
        problem_data_dir: load_env_default("problem_data_dir".to_string(), "data/".to_string()),
        pid_maximum: load_env_default("pid_maximum".to_string(), 4000u32),
        secret_key: Key::generate(),
    }
}