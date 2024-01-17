#![doc = include_str!("../README.md")]

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordVerifier, SaltString},
    Argon2, PasswordHasher,
};
use sha2::{Digest, Sha256};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// SHA hash, often used for registering.
fn sha_hash(plain: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    let mut hasher = Sha256::new();
    hasher.update(plain.as_bytes());
    let res: Vec<u8> = hasher.finalize().to_vec();
    general_purpose::STANDARD.encode(res)
}

/// Convert to md5 hex string. Currently used by [`server::app::user::gravatar`]
pub fn md5_hash(plain: &str) -> String {
    let mut md5 = md5::Md5::new();
    md5.update(plain);
    hex::encode(md5.finalize().as_slice())
}

/// See [`sha_hash`]. Exposed to WASM binary.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn register_hash(plain: &str) -> String {
    sha_hash(plain)
}

/// Hash password with random salt using Argon2 algorithm.
/// Exposed to WASM binary.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn login_hash(plain: &str) -> String {
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(sha_hash(plain).as_bytes(), &salt)
        .unwrap();
    hash.to_string()
}

/// Verify the password.
/// `passwd` is the SHA hash of plain password, `passwd_hash` is the [`login_hash`] of plain password.
/// 
/// View [`server::app::auth::login`] for example.
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
