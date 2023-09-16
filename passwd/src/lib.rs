//! 密码处理模块

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordVerifier, SaltString},
    Argon2, PasswordHasher,
};
use sha2::{Digest, Sha256};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// 注册时将明文密码哈希
fn sha_hash(plain: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    let mut hasher = Sha256::new();
    hasher.update(plain.as_bytes());
    let res: Vec<u8> = hasher.finalize().to_vec();
    general_purpose::STANDARD.encode(res)
}

/// convert to md5 hex string
pub fn md5_hash(plain: &str) -> String {
    let mut md5 = md5::Md5::new();
    md5.update(plain);
    hex::encode(md5.finalize().as_slice())
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn register_hash(plain: &str) -> String {
    sha_hash(plain)
}

/// hash password with random salt
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn login_hash(plain: &str) -> String {
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(sha_hash(plain).as_bytes(), &salt)
        .unwrap();
    hash.to_string()
}

/// verify password
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn verify(passwd: &str, passwd_hash: &str) -> bool {
    if let Ok(hash) = PasswordHash::new(passwd_hash) {
        let argon2 = Argon2::default();
        argon2.verify_password(passwd.as_bytes(), &hash).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_passwd() {
        use super::*;
        let plain = "123546";
        let passwd = register_hash(plain);
        eprintln!("passwd = {}", &passwd);
        let passwd_hash = login_hash(plain);
        eprintln!("passwd_hash = {}", &passwd_hash);
        let passwd_hash = login_hash(plain);
        eprintln!("passwd_hash = {}", &passwd_hash);
        assert!(verify(&passwd, &passwd_hash))
    }
}
