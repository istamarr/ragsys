// Command-line Login Tool
// Supports both SurrealDB and Qdrant credential sources
// Usage: cargo run --bin cmd_login <email> <password>
// # Using SurrealDB (default)
// cargo run --bin cmd_login user@example.com mypassword123

// # Using Qdrant
// CREDENTIAL_SOURCE=qdrant cargo run --bin cmd_login user@example.com mypassword123

use std::env;
use std::collections::HashMap;
use qdrant_client::Qdrant;
// For SurrealDB
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

// For Qdrant
use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::{Condition, Filter, PointId, ScrollPoints, WithPayloadSelector};

// For password decryption
use aes_gcm::{Aes256Gcm, Nonce, Key, KeyInit};
use aes_gcm::aead::Aead;
use base64::decode;

const KEY: [u8; 32] = [0u8; 32]; // 256-bit key (matches secureUtils.rs)

/// Decrypt Base64 encoded data using AES-256-GCM
fn decrypt_data(key: &[u8], encrypted_data: &str) -> Vec<u8> {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
    let encrypted_data = decode(encrypted_data).expect("Invalid base64");
    assert!(encrypted_data.len() >= 12, "Encrypted data too short!");
    let key = Key::<aes_gcm::aes::Aes256>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).expect("Decryption failure!")
}

/// User struct
#[derive(Clone, Debug)]
struct User {
    uid: String,
    email: String,
    password: String,
    role: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("=== Command-line Login Tool ===\n");
        println!("Usage:");
        println!("  cargo run --bin cmd_login <email> <password>\n");
        println!("Example:");
        println!("  cargo run --bin cmd_login user@example.com mypassword123\n");
        println!("Note: Reads CREDENTIAL_SOURCE from environment");
        println!("      CREDENTIAL_SOURCE=surreal (default) or qdrant\n");
        return Ok(());
    }

    let email = &args[1];
    let password = &args[2];

    println!("=== Login Attempt ===");
    println!("Email: {}", email);
    println!("Password: ******\n");

    // Get credential source from environment
    let credential_source = env::var("CREDENTIAL_SOURCE").unwrap_or_else(|_| "surreal".to_string());
    println!("Credential Source: {}\n", credential_source);

    match credential_source.as_str() {
        "qdrant" => login_qdrant(email, password).await,
        _ => login_surreal(email, password).await,
    }
}

/// Login using SurrealDB
async fn login_surreal(email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to SurrealDB...");

    // Get SurrealDB connection details
    let surreal_url = env::var("SURREAL_URL").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let surreal_username = env::var("SURREAL_USERNAME").unwrap_or_else(|_| "root".to_string());
    let surreal_password = env::var("SURREAL_PASSWORD").unwrap_or_else(|_| "root".to_string());
    let surreal_ns = env::var("SURREAL_NS").unwrap_or_else(|_| "pgd_ml_nmspace".to_string());
    let surreal_db = env::var("SURREAL_DB").unwrap_or_else(|_| "pgd_db".to_string());

    let db = Surreal::new::<Ws>(surreal_url).await?;
    db.signin(Root {
        username: &surreal_username,
        password: &surreal_password,
    }).await?;
    db.use_ns(&surreal_ns).use_db(&surreal_db).await?;

    println!("Connected to SurrealDB\n");

    // Query user credentials
    let records: Vec<UserDB> = db.select("user_credentials").await?;

    let mut users: HashMap<String, User> = HashMap::new();
    for user_db in records {
        users.insert(user_db.email.to_string(), User {
            uid: user_db.email.to_string(),
            email: user_db.email.to_string(),
            password: user_db.password.to_string(),
            role: user_db.role.to_string(),
        });
    }

    // Validate credentials
    if let Some(user) = users.get(email) {
        let decrypted_password = String::from_utf8(decrypt_data(&KEY, &user.password))?;

        if decrypted_password == password {
            println!("✓ Login SUCCESSFUL!");
            println!("  Email: {}", user.email);
            println!("  Role: {}", user.role);
            println!("  UID: {}", user.uid);
        } else {
            println!("✗ Login FAILED: Invalid password");
        }
    } else {
        println!("✗ Login FAILED: User not found");
    }

    Ok(())
}

/// Login using Qdrant
async fn login_qdrant(email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Qdrant...");

    let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;

    println!("Connected to Qdrant\n");

    // Query user by email
    let mut filter: Filter = Filter::default();
    filter = Filter::must([
        Condition::matches("email", email.to_string()),
    ]);

    let search_result = client
        .scroll(&ScrollPoints {
            collection_name: "user_credentials".to_string(),
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
        let payload = &point.payload;
        let stored_email = payload.get("email")
            .and_then(|v: &qdrant_client::qdrant::Value| v.as_str())
            .map(|s| s.to_string()).unwrap_or_default();
        let stored_password = payload.get("password")
            .and_then(|v: &qdrant_client::qdrant::Value| v.as_str())
            .map(|s| s.to_string()).unwrap_or_default();
        let stored_role = payload.get("role")
            .and_then(|v: &qdrant_client::qdrant::Value| v.as_str())
            .map(|s| s.to_string()).unwrap_or_default();

        if stored_email == email {
            let decrypted_password = String::from_utf8(decrypt_data(&KEY, &stored_password))?;

            if decrypted_password == password {
                println!("✓ Login SUCCESSFUL!");
                println!("  Email: {}", stored_email);
                println!("  Role: {}", stored_role);
                println!("  UID: {}", stored_email);
            } else {
                println!("✗ Login FAILED: Invalid password");
            }
        } else {
            println!("✗ Login FAILED: User not found");
        }
    } else {
        println!("✗ Login FAILED: User not found");
    }

    Ok(())
}

// SurrealDB UserDB struct
#[derive(Debug, serde::Deserialize)]
struct UserDB {
    #[serde(skip)]
    id: Option<surrealdb::sql::Thing>,
    key: String,
    email: String,
    password: String,
    role: String,
    #[serde(skip)]
    update_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip)]
    create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip)]
    update_by: Option<String>,
    #[serde(skip)]
    create_by: Option<String>,
}
