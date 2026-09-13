// // Password Encryptor/Decryptor for SurrealDB and Qdrant
// // Uses AES-256-GCM encryption with Base64 encoding
// // Based on secureUtils.rs implementation
//
// // # Encrypt a password
// // cargo run --bin password_encryptor encrypt mypassword123
//
// // # Decrypt a Base64 string
// // cargo run --bin password_encryptor decrypt <base64_string>
//
// use aes_gcm::{Aes256Gcm, Nonce, Key, KeyInit};
// use base64::{decode, encode};
// use rand::Rng;
// use std::env;
//
// const KEY: [u8; 32] = [0u8; 32]; // 256-bit key (all zeros, matches secureUtils.rs)
//
// /// Encrypt data using AES-256-GCM and return Base64 encoded string
// fn encrypt_data(key: &[u8], data: &[u8]) -> String {
//     assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
//     let key = Key::from_slice(key);
//     let cipher = Aes256Gcm::new(key);
//     let mut nonce = [0u8; 12]; // 96-bit nonce
//     rand::thread_rng().fill_bytes(&mut nonce);
//
//     let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data).expect("encryption failure!");
//     let encrypted_data = [nonce.to_vec(), ciphertext].concat(); // Prepend nonce for decryption
//     encode(&encrypted_data) // Return the base64-encoded string
// }
//
// /// Decrypt Base64 encoded data using AES-256-GCM
// fn decrypt_data(key: &[u8], encrypted_data: &str) -> Vec<u8> {
//     assert_eq!(key.len(), 32, "Key length must be 32 bytes for AES-256");
//
//     let encrypted_data = decode(encrypted_data).expect("Invalid base64");
//     assert!(encrypted_data.len() >= 12, "Encrypted data too short!");
//
//     let key = Key::from_slice(key);
//     let cipher = Aes256Gcm::new(key);
//
//     let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
//     let nonce = Nonce::from_slice(nonce_bytes);
//
//     cipher.decrypt(nonce, ciphertext).expect("Decryption failure!")
// }
//
// fn main() {
//     let args: Vec<String> = env::args().collect();
//
//     if args.len() < 2 {
//         println!("=== Password Encryptor/Decryptor ===\n");
//         println!("Usage:");
//         println!("  cargo run --bin password_encryptor encrypt <password>");
//         println!("  cargo run --bin password_encryptor decrypt <encrypted_base64>\n");
//         println!("Examples:");
//         println!("  cargo run --bin password_encryptor encrypt mypassword123");
//         println!("  cargo run --bin password_encryptor decrypt <base64_string>\n");
//         println!("Note: Uses AES-256-GCM with 32-byte key (all zeros)");
//         println!("      Matches the implementation in secureUtils.rs\n");
//         return;
//     }
//
//     let command = &args[1];
//
//     match command.as_str() {
//         "encrypt" => {
//             if args.len() < 3 {
//                 println!("Error: Password required for encryption");
//                 println!("Usage: cargo run --bin password_encryptor encrypt <password>");
//                 return;
//             }
//             let password = &args[2];
//             let encrypted = encrypt_data(&KEY, password.as_bytes());
//             println!("Encrypted password (Base64):");
//             println!("{}", encrypted);
//             println!("\nUse this value in SurrealDB or Qdrant 'password' field");
//         }
//         "decrypt" => {
//             if args.len() < 3 {
//                 println!("Error: Encrypted Base64 string required for decryption");
//                 println!("Usage: cargo run --bin password_encryptor decrypt <encrypted_base64>");
//                 return;
//             }
//             let encrypted = &args[2];
//             let decrypted = decrypt_data(&KEY, encrypted);
//             let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8");
//             println!("Decrypted password:");
//             println!("{}", decrypted_str);
//         }
//         _ => {
//             println!("Unknown command: {}", command);
//             println!("Use 'encrypt' or 'decrypt'");
//         }
//     }
// }
fn main(){}
