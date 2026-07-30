use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

const PBKDF2_ITERATIONS: u32 = 600000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn encrypt(plaintext: &str, password: &str, salt: &[u8]) -> String {
    let mut key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .expect("AES-256-GCM requires a 32-byte key");
    let mut nonce_bytes = vec![0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("AES-256-GCM encryption should not fail with valid key and nonce");
    let mut result = Vec::new();
    result.extend_from_slice(salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    key.zeroize();
    BASE64.encode(&result)
}

pub fn decrypt(encoded: &str, password: &str) -> Result<String, String> {
    let data = BASE64.decode(encoded).map_err(|e| e.to_string())?;
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err("Invalid encrypted data".into());
    }
    let salt = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];
    let mut key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .expect("AES-256-GCM requires a 32-byte key");
    let nonce = Nonce::from_slice(nonce_bytes);
    let result = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: wrong password or corrupted data".to_string());
    key.zeroize();
    result.and_then(|plaintext| String::from_utf8(plaintext).map_err(|e| e.to_string()))
}

pub fn encrypt_field(plaintext: &str, password: &str, salt: &[u8]) -> String {
    encrypt(plaintext, password, salt)
}

pub fn decrypt_field(encoded: &str, password: &str) -> Result<String, String> {
    decrypt(encoded, password)
}
