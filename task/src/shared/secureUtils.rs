#[allow(unused)]

use std::convert::Infallible;
use aes::cipher::generic_array::GenericArray;
use aes_gcm::{Aes256Gcm, Nonce, Key, KeyInit,};
use base64::{decode, encode};
use rand::Rng;
use warp::{reply, Reply};
use crate::domain::models::login::{LoginRequest, LoginResponse};
use crate::{Users, WebResult};
use crate::secure::auth;
use crate::secure::error::Error::WrongCredentialsError;
use warp::{reject, Filter,Rejection};
use auth::{with_auth, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use aes_gcm::aead::{Aead,};
use chrono::{DateTime, Utc};
use surrealdb::sql::Thing;
use tokio;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;
use log::info;
use rand::prelude::SliceRandom;
use redis::RedisResult;
use rand::rngs::OsRng;
use rand::RngCore;
use crate::secure::error::Error;

pub fn with_users(users : Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move ||  users.clone())
}

pub async fn login_handler(users : Users, body : LoginRequest ) -> WebResult<impl Reply> {
    info!("Login Handler - Login User");
    let key = [0u8; 32]; // Use a 256-bit key
    //note:
    // let data: &[u8] = body.password.to_string().as_bytes;
    // jgn pakai hasil enkripsi (karena akan selalu berbeda)
    let encrypted_data = encrypt_data(&key, body.password.to_string().as_bytes());
    // println!("encrypted_data {:?}",encrypted_data);
    // pakai hasil dekripsi, hasil akan selalu sama
    match users.iter().find(|(_uid, user)| user.email == body.email &&  std::str::from_utf8(&decrypt_data(&key,user.password.as_str())).expect("Invalid UTF-8") == body.password)
    {
        Some((uid, user)) => {
            let token = auth::create_jwt(&uid, &Role::from_str(&user.role))
                .map_err(|e| reject::custom(e))?;

            Ok(reply::json(&LoginResponse {token }))
        },
        None => Err(reject::custom(WrongCredentialsError)),
    }
}


pub fn encrypt_data(key: &[u8], data: &[u8]) -> String {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
    let key = GenericArray::from_slice(key); // Use GenericArray to create the key
    let cipher = Aes256Gcm::new(key);
    let mut nonce = [0u8; 12]; // Ensure it's 12 bytes
    rand::thread_rng().fill_bytes(&mut nonce);

    // let mut nonce = [1, 2, 3, 4, 5];
    // nonce.shuffle(&mut rand::rng());
    // let nonce: [u8; 12] = rand::rng(); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data).expect("encryption failure!");
    let encrypted_data = [nonce.to_vec(), ciphertext].concat(); // Prepend nonce for decryption
    encode(&encrypted_data) // Return the base64-encoded string
}

// pub fn encrypt_data(key: &[u8], data: &[u8]) -> String {
//     assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
//     let key = GenericArray::from_slice(key); // Use GenericArray to create the key
//     let cipher = Aes256Gcm::new(key);
//
//     // Generate a random 12-byte nonce
//     let mut nonce: [u8; 12] = [0; 12];
//     OsRng.fill(&mut nonce);
//
//     let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
//         .expect("encryption failure!");
//
//     let encrypted_data = [nonce.to_vec(), ciphertext].concat(); // Prepend nonce for decryption
//     encode(&encrypted_data) // Return the base64-encoded string
// }


// pub fn encrypt_data(key: &[u8], data: &[u8]) -> String {
//     assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
//
//     let key = Key::from_slice(key); // Use GenericArray to create the key
//     let cipher = Aes256Gcm::new(key);
//
//     // Generate a random 12-byte nonce using OsRng
//     let mut nonce = [0u8; 12];
//     OsRng.fill_bytes(&mut nonce);
//
//     let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
//         .expect("encryption failure!");
//
//     let encrypted_data = [nonce.to_vec(), ciphertext].concat(); // Prepend nonce for decryption
//     encode(&encrypted_data) // Return the base64-encoded string
// }



// pub fn decrypt_data(key: &[u8], encrypted_data: &str) -> Vec<u8> {
//     assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
//
//     let encrypted_data = decode(encrypted_data).expect("Invalid base64"); // Decode the base64 string
//     let key = GenericArray::from_slice(key); // Use GenericArray to create the key
//     let cipher = Aes256Gcm::new(key);
//     let (nonce, ciphertext) = encrypted_data.split_at(12);
//
//     cipher.decrypt(Nonce::from_slice(nonce), ciphertext).expect("decryption failure!")
// }

// use aes_gcm::aead::{Aead, NewAead, Nonce};
// use aes_gcm::Aes256Gcm;
// use base64::decode;
// use generic_array::GenericArray;

pub fn decrypt_data(key: &[u8], encrypted_data: &str) -> Vec<u8> {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");

    // Decode the base64-encoded data
    let encrypted_data = decode(encrypted_data).expect("Invalid base64");

    // Ensure `encrypted_data` is at least 12 bytes for nonce
    assert!(encrypted_data.len() >= 12, "Encrypted data too short!");

    let key = GenericArray::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Split into nonce (first 12 bytes) and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);

    // Convert nonce bytes into correct format
    let nonce = Nonce::from_slice(nonce_bytes);

    // Attempt decryption
    cipher.decrypt(nonce, ciphertext).expect("Decryption failure!")
}


// apikey:
pub fn generate_api_key() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn store_api_key(key: &str) {

}

pub fn validate_api_key(provided_key: &str, stored_hashed_key: &str) -> bool {
    let hashed_provided_key = hash_api_key(provided_key);
    hashed_provided_key == stored_hashed_key
}

pub fn convert_redis_result(result: redis::RedisResult<String>) -> String {
    match result {
        Ok(value) => value,
        Err(e) => format!("Error: {}", e),
    }
}


#[derive(Debug)]
pub enum ErrMsg {
    Task(crate::secure::error::Error),
    run_debug_err(crate::secure::error::Error),
}
pub struct ErrHandler(ErrMsg);
impl ErrHandler {
    fn err_rag() -> fn(Error) -> ErrMsg {
        ErrMsg::Task
    }
}