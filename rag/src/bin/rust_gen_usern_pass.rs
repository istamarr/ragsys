// Username and password for login
// Based on secureUtils.rs analysis

fn main() {
    println!("=== Credential Source Options ===\n");
    println!("Set environment variable CREDENTIAL_SOURCE to choose:\n");
    println!("  CREDENTIAL_SOURCE=surreal  -> Load from SurrealDB (default)");
    println!("  CREDENTIAL_SOURCE=qdrant   -> Load from Qdrant\n");

    println!("=== Option 1: SurrealDB ===\n");
    println!("=== Database Connection ===\n");
    println!("URL: 127.0.0.1:8000");
    println!("Username: root");
    println!("Password: root");
    println!("Namespace: pgd_ml_nmspace");
    println!("Database: pgd_db\n");

    println!("=== Required Table: user_credentials ===\n");
    println!("Table structure (UserDB model):\n");
    println!("  id: Option<Thing>           // SurrealDB record ID");
    println!("  key: String                  // Unique key");
    println!("  email: String                // User email (used for login)");
    println!("  password: String             // Encrypted password (AES-256-GCM)");
    println!("  role: String                 // User role (e.g., 'DEV', 'ADMIN')");
    println!("  update_date: Option<DateTime<Utc>>");
    println!("  create_date: Option<DateTime<Utc>>");
    println!("  update_by: Option<String>");
    println!("  create_by: Option<String>\n");

    println!("=== Example SurrealDB Query to Create User ===\n");
    println!("USE NS pgd_ml_nmspace DB pgd_db;\n");
    println!("CREATE user_credentials CONTENT {{");
    println!("  key: 'user1',");
    println!("  email: 'user@example.com',");
    println!("  password: '<ENCRYPTED_PASSWORD_BASE64>',");
    println!("  role: 'DEV',");
    println!("  create_date: time::now(),");
    println!("  update_date: time::now()");
    println!("}};\n");

    println!("=== Option 2: Qdrant ===\n");
    println!("=== Database Connection ===\n");
    println!("URL: http://localhost:6333 (from QDRANT_URL env var)");
    println!("Port 6334: http://localhost:6334 (from QDRANT_URL_PORT_6334 env var)\n");

    println!("=== Required Collection: user_credentials ===\n");
    println!("Collection structure (payload fields):\n");
    println!("  key: String                  // Unique key");
    println!("  email: String                // User email (used for login)");
    println!("  password: String             // Encrypted password (AES-256-GCM)");
    println!("  role: String                 // User role (e.g., 'DEV', 'ADMIN')");
    println!("  update_date: String          // ISO datetime");
    println!("  create_date: String          // ISO datetime");
    println!("  update_by: String");
    println!("  create_by: String\n");

    println!("=== Example Qdrant Point to Create User ===\n");
    println!("POST /collections/user_credentials/points?wait=true\n");
    println!("{{");
    println!("  \"points\": [{{");
    println!("    \"id\": 1,");
    println!("    \"vector\": [],  // Empty vector for user credentials");
    println!("    \"payload\": {{");
    println!("      \"key\": \"user1\",");
    println!("      \"email\": \"user@example.com\",");
    println!("      \"password\": \"<ENCRYPTED_PASSWORD_BASE64>\",");
    println!("      \"role\": \"DEV\",");
    println!("      \"create_date\": \"2024-01-01T00:00:00Z\",");
    println!("      \"update_date\": \"2024-01-01T00:00:00Z\"");
    println!("    }}");
    println!("  }}]");
    println!("}}\n");

    println!("=== Password Encryption ===\n");
    println!("Algorithm: AES-256-GCM");
    println!("Key: 32 bytes (currently [0u8; 32] - all zeros)");
    println!("Nonce: 12 bytes (randomly generated per encryption)");
    println!("Encoding: Base64\n");

    println!("=== Note ===\n");
    println!("The password must be encrypted using AES-256-GCM with a 32-byte key");
    println!("before storing in the database. The encrypt_data() function in");
    println!("secureUtils.rs handles this encryption.\n");
}
