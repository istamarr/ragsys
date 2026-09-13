use serde::{Deserialize, Serialize};
use std::env;
use log::{error, info};
use anyhow::{Context, Result};
use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::{Condition, Filter, ScrollPoints, WithPayloadSelector};
use aes_gcm::{Aes256Gcm, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use base64::decode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPayload {
    pub uid: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPoint {
    pub id: String,
    pub payload: UserPayload,
    pub vector: Vec<f32>,
}

fn decrypt_data(key: &[u8], encrypted_data: &str) -> Vec<u8> {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
    let encrypted_data = decode(encrypted_data).expect("Invalid base64");
    assert!(encrypted_data.len() >= 12, "Encrypted data too short!");
    let key = aes_gcm::Key::<aes_gcm::aes::Aes256>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).expect("Decryption failure!")
}

async fn search_user(
    collection_name: &str,
    email: &str,
) -> Result<Option<UserPayload>> {
    let log = "Login Using Qdrant - Search User".to_string();
    info!("{}", log.clone());

    let qdrant_url = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());

    let client = Qdrant::from_url(&*qdrant_url).build()?;

    let filter = Filter::must([
        Condition::matches("email", email.to_string()),
    ]);

    let search_result = client
        .scroll(&ScrollPoints {
            collection_name: collection_name.to_string(),
            filter: Some(filter),
            limit: Some(1),
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                ),
            }),
            ..Default::default()
        })
        .await?;

    if let Some(point) = search_result.result.first() {
        let p = &point.payload;
        let uid:      String = p.get("uid").and_then(|v: &qdrant_client::qdrant::Value| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
        let email:   String = p.get("email").and_then(|v: &qdrant_client::qdrant::Value| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
        let password: String = p.get("password").and_then(|v: &qdrant_client::qdrant::Value| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
        let role:    String = p.get("role").and_then(|v: &qdrant_client::qdrant::Value| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
        let key:     String = p.get("key").and_then(|v: &qdrant_client::qdrant::Value| v.as_str()).map(|s| s.to_string()).unwrap_or_default();

        if !email.is_empty() {
            info!("{} - Found user: {} role: {}", log.clone(), email, role);
            return Ok(Some(UserPayload { uid, email, password, role, key }));
        }
    }

    info!("{} - User not found", log.clone());
    Ok(None)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let log = "Login Using Qdrant".to_string();
    info!("{}", log.clone());

    let collection_name = env::var("CREDENTIAL_COLLECTION")
        .unwrap_or_else(|_| "user_credentials".to_string());

    let input_email    = "istamar.rozid@gmail.com";
    let input_password = "Istamar123!";

    info!("{} - Searching for email: {}", log.clone(), input_email);

    match search_user(&collection_name, input_email).await? {
        Some(user) => {
            let key = [0u8; 32];
            let decrypted = String::from_utf8(decrypt_data(&key, &user.password))
                .unwrap_or_default();

            if decrypted == input_password {
                info!("{} - LOGIN SUCCESS | email: {} | role: {}", log.clone(), user.email, user.role);
                println!("✓ Login SUCCESSFUL!");
                println!("  Email : {}", user.email);
                println!("  Role  : {}", user.role);
                println!("  UID   : {}", user.uid);
            } else {
                info!("{} - LOGIN FAILED: wrong password for {}", log.clone(), user.email);
                println!("✗ Login FAILED: Invalid password");
            }
        }
        None => {
            info!("{} - LOGIN FAILED: user not found", log.clone());
            println!("✗ Login FAILED: User not found");
        }
    }

    info!("{} - Done", log.clone());
    Ok(())
}
